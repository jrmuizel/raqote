use crate::draw_target::*;
use crate::path_builder::*;

#[test]
fn rasterizer() {
    let mut dt = DrawTarget::new(2, 2);
    let mut pb = PathBuilder::new();
    pb.rect(1., 1., 1., 1.);
    dt.fill(&pb.finish(), &Source::Solid(SolidSource{r: 0xff, g: 0xff, b: 0xff, a: 0xff}));
    let white = 0xffffffff;
    assert_eq!(dt.buf, vec![0, 0, 0, white])
}