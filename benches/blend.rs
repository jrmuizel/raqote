use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_raqote(c: &mut Criterion) {
    use raqote::*;

    let mut dt = DrawTarget::new(250, 250);

    let mut pb = PathBuilder::new();
    pb.move_to(10.0, 10.0);
    pb.cubic_to(20.0, 30.0, 120.0, 250.0, 200.0, 150.0);
    pb.close();
    let path = pb.finish();

    let src = Source::from(Color::new(200, 50, 127, 150));

    let draw_opt = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: 1.0,
        antialias: AntialiasMode::None,
    };

    c.bench_function("fill screen raqote", |b| b.iter(|| {
        dt.fill(&path, &src, &draw_opt);
    }));
}

criterion_group!(benches, bench_raqote);
criterion_main!(benches);
