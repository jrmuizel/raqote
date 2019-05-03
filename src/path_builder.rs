use lyon_geom::QuadraticBezierSegment;
use lyon_geom::CubicBezierSegment;
use lyon_geom::Arc;
use euclid::{Point2D, Vector2D, };
use lyon_geom::math::Angle;

#[derive(Clone, Debug)]
pub enum PathOp {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close
}

#[derive(Debug)]
pub struct Path {
    pub ops: Vec<PathOp>
}

impl Path {
    pub fn flatten(&self, tolerance: f32) -> Path {
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

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close();
    }

    pub fn cubic_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
        self.path.ops.push(PathOp::CubicTo(cx1, cy1, cx2, cy2, x, y))
    }

    pub fn close(&mut self) {
        self.path.ops.push(PathOp::Close)
    }

    pub fn arc(&mut self, x: f32, y: f32, r: f32, angle1: f32, angle2: f32) {
        //XXX: handle the current point being the wrong spot
        let a: Arc<f32> = Arc {
            center: Point2D::new(x, y),
            radii: Vector2D::new(r, r),
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