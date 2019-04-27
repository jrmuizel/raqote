use crate::path_builder::*;

use euclid::{Point2D, Vector2D};

type Point = Point2D<f32>;
type Vector = Vector2D<f32>;

use lyon_geom::LineSegment;


fn dash_path(path: &Path, dash_array: &[f32], dash_offset: f32) -> Path {
    let mut dashed = PathBuilder::new();

    let mut cur_x = 0.;
    let mut cur_y = 0.;
    let mut current_dash = 0;
    let mut start_point = Point::zero();
    let mut remaining_dash_length = dash_array[current_dash % dash_array.len()];
    let mut dash_on = true;
    for op in &path.ops {
        match *op {
            PathOp::MoveTo(x, y) => {
                cur_x = x;
                cur_y = y;
                start_point = Point::new(x, y);
                dashed.move_to(x, y);
            }
            PathOp::LineTo(x, y) => {
                let line = LineSegment { from: Point::new(cur_x, cur_y), to: Point::new(x, y)};
                let mut len = line.length();
                let lv = line.to_vector().normalize();
                while len > remaining_dash_length {
                    let seg = lv * remaining_dash_length;
                    if dash_on {
                        dashed.line_to(seg.x, seg.y);
                    } else {
                        dashed.move_to(seg.x, seg.y);
                    }
                    dash_on = !dash_on;
                    current_dash += 1;
                    len -= remaining_dash_length;
                    remaining_dash_length = dash_array[current_dash % dash_array.len()];
                }
                if dash_on {
                    dashed.line_to(x, y);
                } else {
                    dashed.move_to(x, y);
                }
                remaining_dash_length -= len;

                cur_x = x;
                cur_y = y;

            }
            PathOp::Close => {
                let line = LineSegment { from: Point::new(cur_x, cur_y), to: start_point};
                let mut len = line.length();
                let lv = line.to_vector().normalize();
                while len > remaining_dash_length {
                    let seg = lv * remaining_dash_length;
                    if dash_on {
                        dashed.line_to(seg.x, seg.y);
                    } else {
                        dashed.move_to(seg.x, seg.y);
                    }
                    dash_on = !dash_on;
                    current_dash += 1;
                    len -= remaining_dash_length;
                    remaining_dash_length = dash_array[current_dash % dash_array.len()];
                }
                if dash_on {
                    dashed.close();
                }
                remaining_dash_length -= len;
            },
            PathOp::QuadTo(..) => {
                panic!("Only flat paths handled")
            }
            PathOp::CubicTo(..) => {
                panic!("Only flat paths handled")
            }
        }
    }
    dashed.finish()
}