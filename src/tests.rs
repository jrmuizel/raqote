#[cfg(test)]
mod tests {
    use crate::draw_target::*;
    use crate::path_builder::*;
    use crate::stroke::*;
    use crate::Image;

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
        let mut path = pb.finish();
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
}
