use crate::path_builder::*;

use crate::Point;

use lyon_geom::LineSegment;

pub fn dash_path(path: &Path, dash_array: &[f32], mut dash_offset: f32) -> Path {
    let mut dashed = PathBuilder::new();

    let mut cur_pt = Point::zero();
    let mut current_dash = 0;
    let mut start_point = Point::zero();
    let mut remaining_dash_length = dash_array[current_dash % dash_array.len()];
    let mut dash_on = true;

    // adjust our position in the dash array by the dash offset
    while dash_offset > remaining_dash_length {
        dash_offset -= remaining_dash_length;
        current_dash += 1;
        remaining_dash_length = dash_array[current_dash % dash_array.len()];
        dash_on = !dash_on;
    }
    remaining_dash_length -= dash_offset;

    for op in &path.ops {
        match *op {
            PathOp::MoveTo(pt) => {
                cur_pt = pt;
                start_point = pt;
                dashed.move_to(pt.x, pt.y);
            }
            PathOp::LineTo(pt) => {
                let mut start = cur_pt;
                let line = LineSegment {
                    from: start,
                    to: pt,
                };
                let mut len = line.length();
                let lv = line.to_vector().normalize();
                while len > remaining_dash_length {
                    let seg = start + lv * remaining_dash_length;
                    if dash_on {
                        dashed.line_to(seg.x, seg.y);
                    } else {
                        dashed.move_to(seg.x, seg.y);
                    }
                    dash_on = !dash_on;
                    current_dash += 1;
                    len -= remaining_dash_length;
                    remaining_dash_length = dash_array[current_dash % dash_array.len()];
                    start = seg;
                }
                if dash_on {
                    dashed.line_to(pt.x, pt.y);
                } else {
                    dashed.move_to(pt.x, pt.y);
                }
                remaining_dash_length -= len;

                cur_pt = pt;
            }
            PathOp::Close => {
                let mut start = cur_pt;
                let line = LineSegment {
                    from: start,
                    to: start_point,
                };
                let mut len = line.length();
                let lv = line.to_vector().normalize();
                while len > remaining_dash_length {
                    let seg = start + lv * remaining_dash_length;
                    if dash_on {
                        dashed.line_to(seg.x, seg.y);
                    } else {
                        dashed.move_to(seg.x, seg.y);
                    }
                    dash_on = !dash_on;
                    current_dash += 1;
                    len -= remaining_dash_length;
                    remaining_dash_length = dash_array[current_dash % dash_array.len()];
                    start = seg;
                }
                if dash_on {
                    dashed.close();
                }
                remaining_dash_length -= len;
            }
            PathOp::QuadTo(..) => panic!("Only flat paths handled"),
            PathOp::CubicTo(..) => panic!("Only flat paths handled"),
        }
    }
    dashed.finish()
}
