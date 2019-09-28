use crate::path_builder::*;

use crate::Point;

use lyon_geom::LineSegment;

pub fn dash_path(path: &Path, dash_array: &[f32], mut dash_offset: f32) -> Path {
    let mut dashed = PathBuilder::new();

    let mut cur_pt = Point::zero();
    let mut current_dash = 0;
    let mut start_point = Point::zero();

    let mut total_dash_length = 0.;
    for dash in dash_array {
        total_dash_length += *dash;
    }
    if dash_array.len() % 2 == 1 {
        // if the number of items in the dash_array is odd then we need it takes two periods
        // to return to the beginning.
        total_dash_length *= 2.;
    }

    // Handle large positive and negative offsets so that we don't loop for a high number of
    // iterations below in extreme cases
    dash_offset = dash_offset % total_dash_length;
    if dash_offset < 0. {
        dash_offset += total_dash_length;
    }

    let mut remaining_dash_length = dash_array[current_dash % dash_array.len()];
    let mut dash_on = true;

    // To handle closed paths we need a bunch of extra state so that we properly
    // join the first segment. Unfortunately, this makes the code sort of hairy.
    // We need to store all of the points in the initial segment so that we can
    // join the end of the path with it.
    let mut is_first_segment = true;
    let mut first_dash = true;
    let mut initial_segment : Vec<Point> = Vec::new();

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

                // flush the previous initial segment
                if initial_segment.len() > 0 {
                    dashed.move_to(initial_segment[0].x, initial_segment[0].y);
                    for i in 1..initial_segment.len() {
                        dashed.line_to(initial_segment[i].x, initial_segment[i].y);
                    }
                }
                is_first_segment = true;
                initial_segment = Vec::new();
                first_dash = true;
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
                        if is_first_segment {
                            initial_segment.push(start);
                            initial_segment.push(seg);
                        } else {
                            dashed.line_to(seg.x, seg.y);
                        }
                    } else {
                        first_dash = false;
                        dashed.move_to(seg.x, seg.y);
                    }
                    is_first_segment = false;
                    dash_on = !dash_on;
                    current_dash += 1;
                    len -= remaining_dash_length;
                    remaining_dash_length = dash_array[current_dash % dash_array.len()];
                    start = seg;
                }
                if dash_on {
                    if is_first_segment {
                        initial_segment.push(start);
                        initial_segment.push(pt);
                    } else {
                        dashed.line_to(pt.x, pt.y);
                    }
                } else {
                    first_dash = false;
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
                        if is_first_segment {
                            initial_segment.push(start);
                            initial_segment.push(seg);
                        } else {
                            dashed.line_to(seg.x, seg.y);
                        }
                    } else {
                        first_dash = false;
                        dashed.move_to(seg.x, seg.y);
                    }
                    dash_on = !dash_on;
                    current_dash += 1;
                    len -= remaining_dash_length;
                    remaining_dash_length = dash_array[current_dash % dash_array.len()];
                    start = seg;
                }

                if dash_on {
                    if first_dash {
                        // If we're still on the first dash we can just close
                        dashed.close();
                    } else {
                        if initial_segment.len() > 0 {
                            // If have an initial segment we'll need to connect with it
                            for pt in initial_segment {
                                dashed.line_to(pt.x, pt.y);
                            }
                        } else {
                            dashed.line_to(start_point.x, start_point.y);
                        }
                    }
                } else {
                    if initial_segment.len() > 0 {
                        dashed.move_to(initial_segment[0].x, initial_segment[0].y);
                        for i in 1..initial_segment.len() {
                            dashed.line_to(initial_segment[i].x, initial_segment[i].y);
                        }
                    }
                }
                initial_segment = Vec::new();
                remaining_dash_length -= len;
            }
            PathOp::QuadTo(..) => panic!("Only flat paths handled"),
            PathOp::CubicTo(..) => panic!("Only flat paths handled"),
        }
    }

    // We still have an intial segment that we need to emit
    if initial_segment.len() > 0 {
        dashed.move_to(initial_segment[0].x, initial_segment[0].y);
        for i in 1..initial_segment.len() {
            dashed.line_to(initial_segment[i].x, initial_segment[i].y);
        }
    }
    dashed.finish()
}
