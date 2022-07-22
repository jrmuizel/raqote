fn main() {
    use raqote::*;

let mut dt = DrawTarget::new(400, 400);

let mut pb = PathBuilder::new();
pb.rect(0., 0., 400., 400.);
let path = pb.finish();

let gradient = Source::new_sweep_gradient(
    Gradient {
        stops: vec![
            GradientStop {
                position: 0.,
                color: Color::new(0xff, 0, 0, 0),
            },
            GradientStop {
                position: 0.5,
                color: Color::new(0xff, 0xff, 0xff, 0x0),
            },
            GradientStop {
                position: 1.,
                color: Color::new(0xff, 0, 0, 0x0),
            },
        ],
    },
    Point::new(150., 200.),
    45.,
    180.+45.,
    Spread::Repeat,
);
dt.fill(&path, &gradient, &DrawOptions::new());



dt.write_png("example.png");
}