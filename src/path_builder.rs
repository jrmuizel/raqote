use crate::types::Point;
use crate::rasterizer::Rasterizer;
use crate::geom::*;


use lyon_geom::cubic_to_quadratic::cubic_to_quadratics;
use lyon_geom::CubicBezierSegment;
use euclid::Point2D;

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