use lyon_geom::math::Angle;
use lyon_geom::Arc;
use lyon_geom::CubicBezierSegment;
use lyon_geom::QuadraticBezierSegment;

use crate::{Point, Vector};

#[derive(Clone, Debug)]
pub enum PathOp {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}

#[derive(Debug)]
pub struct Path {
    pub ops: Vec<PathOp>,
}

impl Path {
    pub fn flatten(&self, tolerance: f32) -> Path {
        let mut cur_pt = Point::zero();
        let mut flattened = Path { ops: Vec::new() };
        for op in &self.ops {
            match *op {
                PathOp::MoveTo(pt) | PathOp::LineTo(pt) => {
                    cur_pt = pt;
                    flattened.ops.push(op.clone())
                }
                PathOp::Close => flattened.ops.push(op.clone()),
                PathOp::QuadTo(cpt, pt) => {
                    let c = QuadraticBezierSegment {
                        from: cur_pt,
                        ctrl: cpt,
                        to: pt,
                    };
                    for l in c.flattened(tolerance) {
                        flattened.ops.push(PathOp::LineTo(l));
                    }
                    cur_pt = pt;
                }
                PathOp::CubicTo(cpt1, cpt2, pt) => {
                    let c = CubicBezierSegment {
                        from: cur_pt,
                        ctrl1: cpt1,
                        ctrl2: cpt2,
                        to: pt,
                    };
                    for l in c.flattened(tolerance) {
                        flattened.ops.push(PathOp::LineTo(l));
                    }
                    cur_pt = pt;
                }
            }
        }
        flattened
    }
}

pub struct PathBuilder {
    path: Path,
}

impl PathBuilder {
    pub fn new() -> PathBuilder {
        PathBuilder {
            path: Path { ops: Vec::new() },
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.ops.push(PathOp::MoveTo(Point::new(x, y)))
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.ops.push(PathOp::LineTo(Point::new(x, y)))
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path
            .ops
            .push(PathOp::QuadTo(Point::new(cx, cy), Point::new(x, y)))
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close();
    }

    pub fn cubic_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
        self.path.ops.push(PathOp::CubicTo(
            Point::new(cx1, cy1),
            Point::new(cx2, cy2),
            Point::new(x, y),
        ))
    }

    pub fn close(&mut self) {
        self.path.ops.push(PathOp::Close)
    }

    pub fn arc(&mut self, x: f32, y: f32, r: f32, angle1: f32, angle2: f32) {
        //XXX: handle the current point being the wrong spot
        let a: Arc<f32> = Arc {
            center: Point::new(x, y),
            radii: Vector::new(r, r),
            start_angle: Angle::radians(angle1),
            sweep_angle: Angle::radians(angle2),
            x_rotation: Angle::zero(),
        };
        a.for_each_quadratic_bezier(&mut |q| {
            self.quad_to(q.ctrl.x, q.ctrl.y, q.to.x, q.to.y);
        });
    }

    pub fn finish(self) -> Path {
        self.path
    }
}
