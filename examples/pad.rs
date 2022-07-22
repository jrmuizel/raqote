extern crate raqote;

use raqote::*;
use std::fs::*;
use sw_composite::{Gradient, GradientStop, Image};

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
fn main() {
    let mut dt = DrawTarget::new(200, 200);
    
    let gradient = Source::new_linear_gradient(
        Gradient {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: Color::new(0xff, 0xff, 0xff, 0xff),
                },
                GradientStop {
                    position: 0.9999,
                    color: Color::new(0xff, 0x0, 0x0, 0x0),
                },
                GradientStop {
                    position: 1.0,
                    color: Color::new(0xff, 0x0, 0x0, 0x0),
                },
            ],
        },
        Point::new(40., 0.),
        Point::new(100., 0.),
        Spread::Pad,
    );

    let mut pb = PathBuilder::new();
    pb.rect(0., 0., 80., 80.);
    let path = pb.finish();
    dt.fill(&path, &gradient, &DrawOptions::default());

    dt.write_png("out.png").unwrap();
}
