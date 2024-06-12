extern crate raqote;

use raqote::*;
use sw_composite::{Gradient, GradientStop};
use minifb::{Window, WindowOptions, Key};

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

fn main() {
    let mut window = match Window::new("raqote via minifb",
            400, 400,
            WindowOptions {
                resize: false,   // NOTE: minifb appears to have issues with resizing
                ..WindowOptions::default()
            }) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
    };
    
    let mut size = (0, 0);
    println!("Opened window with size {}×{}", size.0, size.1);
    let mut dt = DrawTarget::new(0, 0);
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_size = window.get_size();
        if new_size != size {
            size = new_size;
            println!("Rendering at {}×{}", size.0, size.1);
            dt = DrawTarget::new(size.0 as i32, size.1 as i32);
            render(&mut dt);
        }
        
        if size.0 * size.1 > 0 {
            window.update_with_buffer(dt.get_data()).unwrap();
        } else {
            window.update();
        }
    }
}

fn render(dt: &mut DrawTarget) {
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
    
    let mut pb = PathBuilder::new();
    pb.move_to(100., 10.);
    pb.quad_to(150., 40., 200., 10.);
    pb.quad_to(120., 100., 80., 200.);
    pb.quad_to(150., 180., 200., 200.);
    pb.close();

    pb.move_to(100., 10.);
    pb.cubic_to(150., 40., 175., 0., 200., 10.);
    pb.quad_to(120., 100., 80., 200.);
    pb.quad_to(150., 180., 200., 200.);
    pb.close();

    let path = pb.finish();

    let gradient = Source::RadialGradient(
        Gradient {
            stops: vec![
                GradientStop {
                    position: 0.2,
                    color: 0xff00ff00,
                },
                GradientStop {
                    position: 0.8,
                    color: 0xffffffff,
                },
                GradientStop {
                    position: 1.,
                    color: 0xffff00ff,
                },
            ],
        },
        Spread::Pad,
        Transform::create_translation(-150., -150.),
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
        Point::new(100., 250.),
        &Source::Solid(SolidSource {
            r: 0,
            g: 0,
            b: 0xff,
            a: 0xff,
        }),
        &DrawOptions::new(),
    );
}
