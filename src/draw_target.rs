use crate::rasterizer::Rasterizer;

use crate::blitter::*;
use sw_composite::*;

use crate::types::Point;
use crate::geom::*;
use crate::path_builder::*;
use crate::dash::*;

use lyon_geom::cubic_to_quadratic::cubic_to_quadratics;
use lyon_geom::CubicBezierSegment;
use euclid::Point2D;
use euclid::Transform2D;
use euclid::Box2D;
pub use crate::rasterizer::Winding;

use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};

use std::fs::*;
use std::io::BufWriter;

use png::HasParameters;

use crate::stroke::*;

type Rect = Box2D<i32>;

pub fn rect<T: Copy>(x: T, y: T, w: T, h: T) -> Box2D<T> {
    Box2D::new(Point2D::new(x, y), Point2D::new(w, h))
}

pub struct Mask {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct SolidSource {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub enum Source {
    Solid(SolidSource),
    Image(Image, Transform2D<f32>),
    Gradient(Gradient, Transform2D<f32>)
}

struct Clip {
    rect: Box2D<i32>,
    mask: Option<Vec<u8>>
}

struct Layer {
    buf: Vec<u32>,
    rect: Rect,
}

pub struct DrawTarget {
    width: i32,
    height: i32,
    rasterizer: Rasterizer,
    current_point: Point,
    first_point: Point,
    buf: Vec<u32>,
    clip_stack: Vec<Clip>,
    layer_stack: Vec<Layer>
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
            layer_stack: Vec::new(),
        }
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.current_point = Point { x, y };
        self.first_point = Point { x, y };
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = Point {x, y};
        self.rasterizer.add_edge(self.current_point, p, false, Point {x: 0., y: 0.});
        self.current_point = p;
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let curve = [self.current_point, Point {x: cx, y: cy}, Point { x, y}];
        self.current_point = curve[2];
        self.add_quad(curve);
    }

    fn add_quad(&mut self, mut curve: [Point; 3]) {
        let a = curve[0].y;
        let b = curve[1].y;
        let c = curve[2].y;
        if is_not_monotonic(a, b, c) {
            let mut t_value = 0.;
            if valid_unit_divide(a - b, a - b - b + c, &mut t_value) {
                let mut dst = [Point{ x: 0., y: 0.}; 5];
                chop_quad_at(&curve, &mut dst, t_value);
                flatten_double_quad_extrema(&mut dst);
                self.rasterizer.add_edge(dst[0], dst[2], true, dst[1]);
                self.rasterizer.add_edge(dst[2], dst[4], true, dst[3]);
                return
            }
            // if we get here, we need to force dst to be monotonic, even though
            // we couldn't compute a unit_divide value (probably underflow).
            let b = if abs(a - b) < abs(b - c) { a } else { c };
            curve[1].y = b;
        }
        self.rasterizer.add_edge(curve[0], curve[2], true, curve[1]);

    }

    fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
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

    fn close(&mut self) {
        self.rasterizer.add_edge(self.current_point, self.first_point, false, Point {x: 0., y: 0.});
    }

    pub fn push_clip_rect(&mut self, rect: Rect) {
        // intersect with current clip
        let clip = match self.clip_stack.last() {
            Some(Clip { rect: current_clip, mask: _}) => Clip { rect: current_clip.intersection(&rect), mask: None},
            _ => Clip { rect: rect, mask: None}
        };
        self.clip_stack.push(clip);
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    pub fn push_clip(&mut self, path: &Path) {
        for op in &path.ops {
            match *op {
                PathOp::MoveTo(x, y) => self.move_to(x, y),
                PathOp::LineTo(x, y) => self.line_to(x, y),
                PathOp::QuadTo(cx, cy, x, y) => self.quad_to(cx, cy, x, y),
                PathOp::CubicTo(cx1, cy1, cx2, cy2, x, y) => self.cubic_to(cx1, cy1, cx2, cy2, x, y),
                PathOp::Close => self.close(),
            }
        }

        // XXX: restrict to clipped area
        let mut blitter = MaskSuperBlitter::new(self.width, self.height);
        self.rasterizer.rasterize(&mut blitter, Winding::NonZero);

        if let Some(last) = self.clip_stack.last() {
            // combine with previous mask
            if let Some(last_mask) = &last.mask {
                for i in 0..((self.width * self.height) as usize) {
                    blitter.buf[i] = muldiv255(blitter.buf[i] as u32, last_mask[i] as u32) as u8
                }
            }
        }

        let current_bounds = self.clip_stack.last()
            .map(|c| c.rect)
            .unwrap_or(Box2D::new(Point2D::new(0, 0), Point2D::new(self.width, self.height)));
        //XXX: handle interleaving of clip rect/masks better
        self.clip_stack.push(Clip {
            rect: current_bounds,
            mask: Some(blitter.buf) });
        self.rasterizer.reset();
    }

    pub fn push_layer(&mut self, opacity: f32) {
        unimplemented!()
    }

    pub fn pop_layer(&mut self) {
        unimplemented!()
    }

    pub fn mask(&mut self, src: &Source, x:i32, y: i32, mask: &Mask) {
        self.composite(src, &mask.data, rect(0, 0, mask.width, mask.height));
    }

    pub fn stroke(&mut self, path: &Path, style: &StrokeStyle, src: &Source) {
        let mut path = path.flatten(0.1);
        if !style.dash_array.is_empty() {
            path = dash_path(&path, &style.dash_array, style.dash_offset);
        }
        let stroked = stroke_to_path(&path, style);
        self.fill(&stroked, src, Winding::NonZero);
    }

    pub fn fill(&mut self, path: &Path, src: &Source, winding_mode: Winding) {
        for op in &path.ops {
            match *op {
                PathOp::MoveTo(x, y) => self.move_to(x, y),
                PathOp::LineTo(x, y) => self.line_to(x, y),
                PathOp::QuadTo(cx, cy, x, y) => self.quad_to(cx, cy, x, y),
                PathOp::CubicTo(cx1, cy1, cx2, cy2, x, y) => self.cubic_to(cx1, cy1, cx2, cy2, x, y),
                PathOp::Close => self.close(),
            }
        }
        let mut blitter = MaskSuperBlitter::new(self.width, self.height);
        self.rasterizer.rasterize(&mut blitter, winding_mode);
        self.composite(src, &blitter.buf, rect(0, 0, self.width, self.height));
        self.rasterizer.reset();
    }

    pub fn draw_text(&mut self, font: &Font, point_size: f32, text: &str, mut start: Point2D<f32>, src: &Source) {
        let mut ids = Vec::new();
        let mut positions = Vec::new();
        for c in text.chars() {
            let id = font.glyph_for_char(c).unwrap();
            ids.push(id);
            positions.push(start);
            start += font.advance(id).unwrap() / 96.;
        }
        self.draw_glyphs(font, point_size, &ids, &positions, src);
    }

    pub fn draw_glyphs(&mut self, font: &Font, point_size: f32, ids: &[u32], positions: &[Point2D<f32>], src: &Source) {
        let mut combined_bounds = euclid::Rect::zero();
        for (id, position) in ids.iter().zip(positions.iter()) {
            let bounds = font.raster_bounds(*id, point_size, position, HintingOptions::None,
                                   RasterizationOptions::GrayscaleAa);
            combined_bounds = match bounds {
                Ok(bounds) => { dbg!(position); dbg!(bounds); combined_bounds.union(&bounds) },
                _ => panic!()
            }
        }

        dbg!(combined_bounds);

        /*let mut canvas = Canvas::new(&euclid::Size2D::new(combined_bounds.size.width as u32,
                                     combined_bounds.size.height as u32), Format::A8);*/
        let mut canvas = Canvas::new(&euclid::Size2D::new(self.width as u32,
                                                          self.height as u32), Format::A8);
        for (id, position) in ids.iter().zip(positions.iter()) {
            font.rasterize_glyph(&mut canvas, *id, point_size, position, HintingOptions::None,
                                            RasterizationOptions::GrayscaleAa);
        }

        self.composite(src, &canvas.pixels, rect(0, 0, canvas.size.width as i32, canvas.size.height as i32));
    }

    fn composite(&mut self, src: &Source, mask: &[u8], mut rect: Rect) {
        let mut shader: &Shader;
        let cs;
        let is;
        let gs;
        let image = match src {
            Source::Solid(c) => {
                let color = ((c.a as u32) << 24) |
                    ((c.r as u32) << 16) |
                    ((c.g as u32) << 8) |
                    ((c.b as u32) << 0);
                cs = SolidShader { color };
                shader = &cs;
            }
            Source::Image(ref image, transform) => {
                is = ImageShader::new(image, transform);
                shader = &is;
            }
            Source::Gradient(ref gradient, transform) => {
                gs = GradientShader::new(gradient, transform);
                shader = &gs;
            }
        };

        let mut blitter: &mut Blitter;
        let mut scb;
        let mut sb;
        match self.clip_stack.last() {
            Some(Clip { rect: _, mask: Some(clip)}) => {
                scb = ShaderClipBlitter {
                    shader: shader,
                    tmp: vec![0; self.width as usize],
                    dest: &mut self.buf,
                    dest_stride: self.width,
                    mask,
                    mask_stride: self.width,
                    clip,
                    clip_stride: self.width};

                blitter = &mut scb;
            }
            Some(Clip { rect: clip_rect, mask: _ }) => {
                rect = rect.intersection(clip_rect);
                if rect.is_negative() {
                    return;
                }
                sb = ShaderBlitter {
                    shader: &*shader,
                    tmp: vec![0; self.width as usize],
                    dest: &mut self.buf,
                    dest_stride: self.width,
                    mask,
                    mask_stride: self.width
                };
                blitter = &mut sb;
            }
            _ => {
                sb = ShaderBlitter {
                    shader: &*shader,
                    tmp: vec![0; self.width as usize],
                    dest: &mut self.buf,
                    dest_stride: self.width,
                    mask,
                    mask_stride: self.width
                };
                blitter = &mut sb;
            }
        }

        for y in rect.min.y..rect.max.y {
            blitter.blit_span(y, rect.min.x, rect.max.x);
        }
    }

    pub fn get_data(&self) -> &[u32] {
        &self.buf
    }

    pub fn write_png<P: std::convert::AsRef<std::path::Path>>(&self, path: P) {
        let file = File::create(path).unwrap();

        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let mut output = Vec::with_capacity(self.buf.len()*4);

        for pixel in &self.buf {
            let a = (pixel >> 24) & 0xffu32;
            let mut r = (pixel >> 16) & 0xffu32;
            let mut g = (pixel >> 8) & 0xffu32;
            let mut b = (pixel >> 0) & 0xffu32;

            if a > 0u32 {
                r = r * 255u32 / a;
                g = g * 255u32 / a;
                b = b * 255u32 / a;
            }

            output.push(r as u8);
            output.push(g as u8);
            output.push(b as u8);
            output.push(a as u8);
        }

        writer.write_image_data(&output).unwrap();
    }
}
