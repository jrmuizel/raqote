use crate::path_builder::{Path, PathOp};

pub enum LineCap {
    Round,
    Square,
    Butt
}

pub enum LineJoin {
    Round,
    Mitre,
    Bevel,
}

fn stroke_closed_subpath(dest: &mut Vec<PathOp>, subpath: &[PathOp], width: f32) {

}

fn stroke_open_subpath(dest: &mut Vec<PathOp>, subpath: &[PathOp], width: f32) {
    let mut start_x = 0.;
    let mut start_y = 0.;
    let mut cur_x = 0.;
    let mut cur_y = 0.;
    /*for op in &subpath {
        match *op {
            PathOp::MoveTo(x, y) => {
                start_x = x,
                start_y = y,
                cur_x = x,
                cur_y = y;
            }
            PathOp::LineTo(x, y) => {
                dest.push(PathOp::LineTo())
                cur_x = x;
                cur_y = y;
            }
            _ => panic!()
        }
    }
    cap(dest, LineCap::Butt);

    cap(dest, LineCap::Butt);*/
}

pub fn stroke_to_path(path: &Path, width: f32) -> Path {
    let i = 0;
    let len = path.ops.len();
    let mut cur_x = 0.;
    let mut cur_y = 0.;
    let mut sub_path_start = 0;
    let mut sub_path_end = 0;
    let mut stroked_path = Vec::new();
    for op in &path.ops {
        match *op {
            PathOp::MoveTo(..) => {
                if sub_path_start != sub_path_end {
                    stroke_open_subpath(&mut stroked_path,
                                        &path.ops[sub_path_start..sub_path_end], width);
                }
                sub_path_start = sub_path_end;
            }
            PathOp::LineTo(..) => {
                sub_path_end += 1;
            }
            PathOp::Close => {
                stroke_closed_subpath(&mut stroked_path,
                                      &path.ops[sub_path_start..sub_path_end], width);
                sub_path_end += 1;
                sub_path_start = sub_path_end
            },
            PathOp::QuadTo(..) => {
                panic!("Only flat paths handled")
            }
            PathOp::CubicTo(..) => {
                panic!("Only flat paths handled")
            }
        }
    }
    panic!()
}