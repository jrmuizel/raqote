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
    pb.move_to(340., 190.);
    pb.arc(160., 190., 180., 0., 2. * 3.14159);
    pb.close();
    let path = pb.finish();
    dt.push_clip(&path);

    let mut pb = PathBuilder::new();
    pb.move_to(0., 0.);
    pb.line_to(200., 0.);
    pb.line_to(200., 300.);
    pb.line_to(0., 300.);
    pb.close();
    let path = pb.finish();
    dt.fill(
        &path,
        &Source::Solid(SolidSource {
            r: 0x80,
            g: 0x80,
            b: 0,
            a: 0x80,
        }),
        &DrawOptions::new(),
    );

    let mut pb = PathBuilder::new();
    pb.move_to(50., 50.);
    pb.line_to(100., 70.);
    pb.line_to(110., 150.);
    pb.line_to(40., 180.);
    pb.close();

    /*
    dt.move_to(100., 10.);
    dt.quad_to(150., 40., 200., 10.);
    dt.quad_to(120., 100., 80., 200.);
    dt.quad_to(150., 180., 200., 200.);
    dt.close();
    */

    pb.move_to(100., 10.);
    pb.cubic_to(150., 40., 175., 0., 200., 10.);
    pb.quad_to(120., 100., 80., 200.);
    pb.quad_to(150., 180., 200., 200.);
    pb.close();

    let path = pb.finish();

    let decoder = png::Decoder::new(File::open("photo.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    println!("{:?}", info.color_type);

    let mut image: Vec<u32> = Vec::new();
    for i in buf.chunks(3) {
        image.push(0xff << 24 | ((i[0] as u32) << 16) | ((i[1] as u32) << 8) | (i[2] as u32))
    }
    let _bitmap = Image {
        width: info.width as i32,
        height: info.height as i32,
        data: &image[..],
    };

    //dt.fill(Source::Solid(SolidSource{r: 0xff, g: 0xff, b: 0, a: 0xff}));
    //dt.fill(Source::Bitmap(bitmap, Transform::create_scale(2., 2.)));

    let gradient = Source::RadialGradient(
        Gradient {
            stops: vec![
                GradientStop {
                    position: 0.2,
                    color: Color::new(0xff, 0x00, 0xff, 0x00),
                },
                GradientStop {
                    position: 0.8,
                    color: Color::new(0xff, 0xff, 0xff, 0xff),
                },
                GradientStop {
                    position: 1.,
                    color: Color::new(0xff, 0xff, 0x00, 0xff),
                },
            ],
        },
        Spread::Pad,
        Transform::translation(-150., -150.),
    );
    dt.fill(&path, &gradient, &DrawOptions::new());

    let mut pb = PathBuilder::new();
    pb.move_to(200., 200.);
    pb.line_to(300., 300.);
    pb.line_to(200., 300.);

    let path = pb.finish();
    dt.stroke(
        &path,
        &gradient,
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

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();

    dt.draw_text(
        &font,
        24.,
        "Hello",
        Point::new(0., 100.),
        &Source::Solid(SolidSource {
            r: 0,
            g: 0,
            b: 0xff,
            a: 0xff,
        }),
        &DrawOptions::new(),
    );

    dt.write_png("out.png").unwrap();
}
