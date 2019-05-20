## raqote
[![Build Status](https://travis-ci.org/jrmuizel/raqote.svg?branch=master)](https://travis-ci.org/jrmuizel/raqote)
[![Crates.io](https://img.shields.io/crates/v/raqote.svg)](https://crates.io/crates/raqote)
[![Documentation](https://docs.rs/raqote/badge.svg)](https://docs.rs/raqote)


A pure Rust 2D Graphics Library.

Raqote aims to be a small, simple, fast software 2D graphics library with roughly
the same capabilities as the cairo image backend.

Current functionality
 - path filling
 - stroking
 - dashing
 - image, solid, and gradient fills
 - rectangular and path clipping
 - blend modes
 - layers
 - repeat modes for images
 - global alpha

Planned functionality
 - extended blend modes
 - two circle radial gradients
 - perspective image drawing
 - shadows?

Example:

```rust
use raqote::*;

let mut dt = DrawTarget::new(400, 400);

let mut pb = PathBuilder::new();
pb.move_to(100., 10.);
pb.cubic_to(150., 40., 175., 0., 200., 10.);
pb.quad_to(120., 100., 80., 200.);
pb.quad_to(150., 180., 300., 300.);
pb.close();
let path = pb.finish();

let gradient = Source::new_radial_gradient(
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
    Point::new(150., 150.),
    128.,
    Spread::Pad,
);
dt.fill(&path, &gradient, &DrawOptions::new());

let mut pb = PathBuilder::new();
pb.move_to(100., 100.);
pb.line_to(300., 300.);
pb.line_to(200., 300.);
let path = pb.finish();

dt.stroke(
    &path,
    &Source::Solid(SolidSource {
        r: 0x0,
        g: 0x0,
        b: 0x80,
        a: 0x80,
    }),
    &StrokeStyle {
        cap: LineCap::Round,
        join: LineJoin::Round,
        width: 10.,
        miter_limit: 2.,
        dash_array: vec![10., 18.],
        dash_offset: 16.,
    },
    &DrawOptions::new()
);

dt.write_png("example.png");
```

Produces:

![example.png](https://github.com/jrmuizel/raqote/raw/master/example.png)

