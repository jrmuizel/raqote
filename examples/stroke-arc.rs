use lyon_geom::Transform;
use raqote::*;

fn main() {
    let mut dt = DrawTarget::new(400, 400);

    let mut pb = PathBuilder::new();
    pb.arc(0., 0., 20., 0., std::f32::consts::PI);

    let path = pb.finish();
    dt.set_transform(&Transform::translation(50., 50.));
    dt.stroke(
        &path,
        &Source::Solid(SolidSource::from_unpremultiplied_argb(0xFF, 0, 0x80, 0)),
        &StrokeStyle {
            width: 40., // <--
            ..StrokeStyle::default()
        },
        &DrawOptions::new(),
    );

    dt.write_png("out.png").unwrap();
}