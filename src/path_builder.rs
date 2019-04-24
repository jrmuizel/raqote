use crate::types::Point;
use crate::rasterizer::Rasterizer;
use crate::geom::*;


use lyon_geom::cubic_to_quadratic::cubic_to_quadratics;
use lyon_geom::QuadraticBezierSegment;
use lyon_geom::CubicBezierSegment;
use euclid::Point2D;

#[derive(Clone)]
pub enum PathOp {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close
}

pub struct Path {
    pub ops: Vec<PathOp>
}

impl Path {
    fn flatten(&self, tolerance: f32) -> Path {
        let mut cur_x: f32 = 0.;
        let mut cur_y: f32 = 0.;
        let mut flattened = Path { ops: Vec::new() };
        for op in &self.ops {
            match *op {
                PathOp::MoveTo(x, y) |
                PathOp::LineTo(x, y) => {
                    cur_x = x;
                    cur_y = y;
                    flattened.ops.push(op.clone())
                }
                PathOp::Close => flattened.ops.push(op.clone()),
                PathOp::QuadTo(cx, cy, x, y) => {
                    let c = QuadraticBezierSegment {
                        from: Point2D::new(cur_x, cur_y),
                        ctrl: Point2D::new(cx, cy),
                        to: Point2D::new(x, y)
                    };
                    for l in c.flattened(tolerance) {
                        flattened.ops.push(PathOp::LineTo(l.x, l.y));
                    }
                    cur_x = x;
                    cur_y = y;
                }
                PathOp::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                    let c = CubicBezierSegment {
                        from: Point2D::new(cur_x, cur_y),
                        ctrl1: Point2D::new(c1x, c1y),
                        ctrl2: Point2D::new(c2x, c2y),
                        to: Point2D::new(x, y)
                    };
                    for l in c.flattened(tolerance) {
                        flattened.ops.push(PathOp::LineTo(l.x, l.y));
                    }
                    cur_x = x;
                    cur_y = y;
                }
            }
        }
        flattened
    }
}

pub struct PathBuilder {
    path: Path
}

impl PathBuilder {
    pub fn new() -> PathBuilder {
        PathBuilder {path: Path {ops:Vec::new()}}
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.ops.push(PathOp::MoveTo(x, y))
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.ops.push(PathOp::LineTo(x, y))
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path.ops.push(PathOp::QuadTo(cx, cy, x, y))
    }

    pub fn cubic_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
        self.path.ops.push(PathOp::CubicTo(cx1, cy1, cx2, cy2, x, y))
    }

    pub fn close(&mut self) {
        self.path.ops.push(PathOp::Close)
    }

    pub fn finish(self) -> Path {
        self.path
    }

}