use font_kit::family_name::FamilyName;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use raqote::*;

fn main() {
    let mut dt = DrawTarget::new(300, 100);
    dt.clear(SolidSource::from_unpremultiplied_argb(
        0xff, 0xcf, 0xcf, 0xcf,
    ));

    let font = SystemSource::new()
        .select_best_match(
            &[FamilyName::Title("Roboto".into())],
            &Properties::new().weight(Weight::MEDIUM),
        )
        .unwrap()
        .load()
        .unwrap();
    println!("{:?}", font);

    //dt.set_transform(&Transform::create_translation(50.0, 0.0));
    dt.set_transform(&Transform::rotation(euclid::Angle::degrees(15.0)));
    let font = font_kit::loader::Loader::from_file(&mut std::fs::File::open("res/Box3.ttf").unwrap(), 0).unwrap();
    dt.draw_text(
        &font,
        30.,
        "3",
        Point::new(0., 30.),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 0, 180, 0)),
        &DrawOptions::new(),
    );
    dt.fill_rect(0., 35., 40., 5., &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 0, 180, 0)),
    &DrawOptions::new() );

    dt.write_png("out.png").unwrap();
}
