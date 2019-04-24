use crate::rasterizer::Rasterizer;

use crate::blitter::MaskSuperBlitter;
use sw_composite::*;

use crate::types::Point;
use crate::geom::*;

use lyon_geom::cubic_to_quadratic::cubic_to_quadratics;
use lyon_geom::CubicBezierSegment;
use euclid::Point2D;
use euclid::Transform2D;
use euclid::Box2D;

type Rect = Box2D<i32>;

pub struct SolidSource {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub enum Source {
    Solid(SolidSource),
    Bitmap(Bitmap, Transform2D<f32>),
    Gradient(Gradient, Transform2D<f32>)
}

struct Clip {
    rect: Box2D<i32>
}

pub struct DrawTarget {
    width: i32,
    height: i32,
    rasterizer: Rasterizer,
    current_point: Point,
    first_point: Point,
    pub buf: Vec<u32>,
    clip_stack: Vec<Clip>
}

impl DrawTarget {
    pub fn new(width: i32, height: i32) -> DrawTarget {
        DrawTarget {
            width,
            height,
            current_point: Point { x: 0., y: 0.},
            first_point: Point { x: 0., y: 0. },
            rasterizer: Rasterizer::new(width, height),
            buf: vec![0; (width*height) as usize],
            clip_stack: Vec::new(),
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.current_point = Point { x, y };
        self.first_point = Point { x, y };
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        let p = Point {x, y};
        self.rasterizer.add_edge(self.current_point, p, false, Point {x: 0., y: 0.});
        self.current_point = p;
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let mut curve = [self.current_point, Point {x: cx, y: cy}, Point { x, y}];
        self.current_point = curve[2];
        self.add_quad(curve);
    }

    fn add_quad(&mut self, mut curve: [Point; 3]) {
        let a = curve[0].y;
        let b = curve[1].y;
        let c = curve[2].y;
        if is_not_monotonic(a, b, c) {
            let mut tValue = 0.;
            if valid_unit_divide(a - b, a - b - b + c, &mut tValue) {
                let mut dst = [Point{ x: 0., y: 0.}; 5];
                chop_quad_at(&curve, &mut dst, tValue);
                flatten_double_quad_extrema(&mut dst);
                self.rasterizer.add_edge(dst[0], dst[2], true, dst[1]);
                self.rasterizer.add_edge(dst[2], dst[4], true, dst[3]);
                return
            }
            // if we get here, we need to force dst to be monotonic, even though
            // we couldn't compute a unit_divide value (probably underflow).
            let b = if Sk2ScalarAbs(a - b) < Sk2ScalarAbs(b - c) { a } else { c };
            curve[1].y = b;
        }
        self.rasterizer.add_edge(curve[0], curve[2], true, curve[1]);

    }

    pub fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        let c = CubicBezierSegment {
            from: Point2D::new(self.current_point.x, self.current_point.y),
            ctrl1: Point2D::new(c1x, c1y),
            ctrl2: Point2D::new(c2x, c2y),
            to: Point2D::new(x, y)
        };
        cubic_to_quadratics(&c, 0.01, &mut|q| {
            fn e2r(p: Point2D<f32>) -> Point {
                Point{ x: p.x, y: p.y }
            }
            let curve = [e2r(q.from), e2r(q.ctrl), e2r(q.to)];
            self.add_quad(curve);
        });
        self.current_point = Point { x, y };
    }

    pub fn close(&mut self) {
        self.rasterizer.add_edge(self.current_point, self.first_point, false, Point {x: 0., y: 0.});
    }

    pub fn push_clip_rect(&mut self, rect: Rect) {
        self.clip_stack.push(Clip {rect });
    }
    
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    pub fn fill(&mut self, src: Source) {
        let mut blitter = MaskSuperBlitter::new(self.width, self.height);
        self.rasterizer.rasterize(&mut blitter);

        let color = match src {
            Source::Solid(c) => {
                let color = ((c.a as u32) << 24) |
                    ((c.r as u32) << 16) |
                    ((c.g as u32) << 8) |
                    ((c.b as u32) << 0);
                for i in 0..((self.width*self.height) as usize) {
                    self.buf[i] = over_in(color, self.buf[i], blitter.buf[i] as u32)
                }
            },
            Source::Bitmap(ref bitmap, transform) => {
                let tfm = MatrixFixedPoint {
                    // Is the order right?
                    xx: float_to_fixed(transform.m11),
                    xy: float_to_fixed(transform.m12),
                    yx: float_to_fixed(transform.m21),
                    yy: float_to_fixed(transform.m22),
                    x0: float_to_fixed(transform.m31),
                    y0: float_to_fixed(transform.m32)
                };
                for y in 0..self.height {
                    for x in 0..self.width {
                        let p = tfm.transform(x as u16, y as u16);
                        let color = fetch_bilinear(bitmap, p.x, p.y);
                        self.buf[(y* self.width + x) as usize]
                            = over_in(color,
                                      self.buf[(y* self.width + x) as usize],
                                      blitter.buf[(y* self.width + x) as usize] as u32);
                    }
                }
            }
            Source::Gradient(ref gradient, transform) => {
                let tfm = MatrixFixedPoint {
                    // Is the order right?
                    xx: float_to_fixed(transform.m11),
                    xy: float_to_fixed(transform.m12),
                    yx: float_to_fixed(transform.m21),
                    yy: float_to_fixed(transform.m22),
                    x0: float_to_fixed(transform.m31),
                    y0: float_to_fixed(transform.m32)
                };
                let gs = gradient.make_source(&tfm);
                for y in 0..self.height {
                    for x in 0..self.width {
                        let color = gs.radial_gradient_eval(x as u16, y as u16);
                        self.buf[(y* self.width + x) as usize]
                            = over_in(color,
                                      self.buf[(y* self.width + x) as usize],
                                      blitter.buf[(y* self.width + x) as usize] as u32);
                    }
                }
            }
        };

        self.rasterizer.reset();
    }
}