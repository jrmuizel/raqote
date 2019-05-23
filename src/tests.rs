#[cfg(test)]
mod tests {

    use crate::*;
    use crate::draw_target::intrect;

    #[test]
    fn basic_rasterizer() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        pb.rect(1., 1., 1., 1.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn implict_close() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        pb.move_to(1., 1.);
        pb.line_to(2., 1.);
        pb.line_to(2., 2.);
        pb.line_to(1., 2.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn offscreen_edges() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        pb.rect(1., 0., 8., 1.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, white, 0, 0][..])
    }

    #[test]
    fn clip_rect() {
        let mut dt = DrawTarget::new(2, 2);
        dt.push_clip_rect(intrect(1, 1, 2, 2));
        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 2., 2.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn nested_clip_rect() {
        let mut dt = DrawTarget::new(2, 2);
        dt.push_clip_rect(intrect(0, 1, 2, 2));
        dt.push_clip_rect(intrect(1, 0, 2, 2));
        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 2., 2.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn even_odd_rect() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 2., 2.);
        pb.rect(0., 0., 2., 2.);
        pb.rect(1., 1., 2., 2.);
        let mut path = pb.finish();
        path.winding = Winding::EvenOdd;
        dt.fill(
            &path,
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn clear() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 2., 2.);
        let path = pb.finish();
        dt.fill(
            &path,
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        dt.clear(SolidSource { r: 0, g: 0, b: 0, a: 0 });
        assert_eq!(dt.get_data(), &vec![0, 0, 0, 0][..])
    }

    #[test]
    fn basic_push_layer() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        dt.push_clip_rect(intrect(1, 1, 2, 2));
        dt.push_layer(1.);
        pb.rect(1., 1., 1., 1.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let white = 0xffffffff;
        dt.pop_layer();
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn basic_draw_image() {
        let mut dt = DrawTarget::new(2, 2);
        let mut dt2 = DrawTarget::new(1, 1);

        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 1., 1.);
        dt2.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let image = Image {width: 1, height: 1, data: dt2.get_data()};
        dt.draw_image_at(1., 1., &image, &DrawOptions::default());
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![0, 0, 0, white][..])
    }

    #[test]
    fn repeating_draw_image() {
        let mut dt = DrawTarget::new(4, 1);
        let mut dt2 = DrawTarget::new(2, 1);

        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 1., 1.);
        dt2.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions::new(),
        );
        let image = Image {width: 2, height: 1, data: dt2.get_data()};
        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 4., 1.);
        let source = Source::Image(image,
                                   ExtendMode::Repeat,
                                   Transform::create_translation(0., 0.));

        dt.fill(&pb.finish(), &source, &DrawOptions::default());
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![white, 0, white, 0][..])
    }

    // This test is disable on miri because miri doesn't support hypot()
    // https://github.com/rust-lang/miri/issues/667
    #[cfg(not(miri))]
    #[test]
    fn stroke() {
        let mut dt = DrawTarget::new(3, 3);
        let mut pb = PathBuilder::new();
        pb.rect(0.5, 0.5, 2., 2.);
        dt.stroke(&pb.finish(), &Source::Solid(SolidSource {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            a: 0xff,
        }),
                  &StrokeStyle { width: 1., ..Default::default()},
                  &DrawOptions::new());
        let white = 0xffffffff;
        assert_eq!(dt.get_data(), &vec![white, white, white,
                                        white, 0, white,
                                        white, white, white][..])
    }

    #[cfg(not(miri))]
    #[test]
    fn degenerate_stroke() {
        let mut dt = DrawTarget::new(3, 3);
        let mut pb = PathBuilder::new();
        pb.move_to(0.5, 0.5);
        pb.line_to(2., 2.);
        pb.line_to(2., 2.);
        pb.line_to(4., 2.);
        dt.stroke(&pb.finish(), &Source::Solid(SolidSource {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            a: 0xff,
        }),
                  &StrokeStyle { width: 1., ..Default::default()},
                  &DrawOptions::new());

    }

    #[cfg(not(miri))]
    #[test]
    fn dashing() {
        let mut dt = DrawTarget::new(3, 3);
        let mut pb = PathBuilder::new();
        pb.move_to(40., 40.);
        pb.line_to(160., 40.);
        pb.line_to(160., 160.);
        pb.line_to(160., 160.);
        pb.close();
        dt.stroke(&pb.finish(), &Source::Solid(SolidSource {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            a: 0xff,
        }),
                  &StrokeStyle { width: 1., dash_array: vec![10.0, 6.0, 4.0, 10.0, 6.0, 4.0],
                      dash_offset: 15.0, ..Default::default()},
                  &DrawOptions::new());

    }

    #[test]
    fn draw_options_alpha() {
        let mut dt = DrawTarget::new(2, 2);
        let mut pb = PathBuilder::new();
        pb.rect(1., 1., 1., 1.);
        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),
            &DrawOptions {
                blend_mode: BlendMode::SrcOver,
                alpha: 0.,
            }
        );
        assert_eq!(dt.get_data(), &vec![0, 0, 0, 0][..])
    }

    #[test]
    fn blend_zero() {
        let mut dt = DrawTarget::new(2, 2);

        dt.clear(SolidSource {
            r: 0xff,
            g: 0xff,
            b: 0xff,
            a: 0xff,
        });

        let source = Source::Solid(SolidSource {
            r: 0x00,
            g: 0x00,
            b: 0x00,
            a: 0xff,
        });

        let mut pb = PathBuilder::new();
        pb.rect(0., 0., 1., 1.);
        let path = pb.finish();
        dt.fill(&path, &source, &DrawOptions::default());
        let white = 0xffffffff;
        let black = 0xff000000;

        assert_eq!(dt.get_data(), &vec![black, white, white, white][..])

    }
}
