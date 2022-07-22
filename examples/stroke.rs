use raqote::*;

fn main() {
    let mut dt = DrawTarget::new(400, 400);

    let mut pb = PathBuilder::new();
    pb.move_to(200., 200.);
    pb.line_to(300., 300.);
    pb.line_to(200., 300.);

    let path = pb.finish();
    dt.stroke(
        &path,
        &Source::Solid(SolidSource::from_unpremultiplied_argb(0xFF, 0, 0x80, 0)),
        &StrokeStyle {
            width: 100000., // <--
            ..StrokeStyle::default()
        },
        &DrawOptions::new(),
    );

    dt.write_png("out.png").unwrap();
}