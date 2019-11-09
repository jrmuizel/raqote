extern crate raqote;

use raqote::*;
use std::fs::*;
use sw_composite::{Gradient, GradientStop, Image};

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

fn main() {
    let mut dt = DrawTarget::new(400, 400);



    let mut pb = PathBuilder::new();
    pb.line_to(0., 100.);
    pb.line_to(100., 100.);

    //pb.arc(200., 25., 5., 0., 2.*3.14159);
    let path = pb.finish();
    dt.fill(
        &path,
        &Source::Solid(SolidSource {
            r: 0xff,
            g: 0,
            b: 0xff,
            a: 0xff,
        }),

        &DrawOptions::new(),
    );
    dt.stroke(
        &path,
        &Source::Solid(SolidSource {
            r: 0,
            g: 0,
            b: 0xff,
            a: 0xff,
        }),
        &StrokeStyle {
            cap: LineCap::Butt,
            join: LineJoin::Bevel,
            width: 10.,
            miter_limit: 2.,
            dash_array: vec![10., 5.],
            dash_offset: 3.,
        },
        &DrawOptions::new(),
    );



    dt.write_png("out.png").unwrap();
}
