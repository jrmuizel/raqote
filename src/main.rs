mod rasterizer;
mod types;
mod geom;

use typed_arena::Arena;
use types::Point;
use std::fs::*;
use std::io::BufWriter;

mod path_builder;
use path_builder::PathBuilder;

use png::HasParameters;


pub fn unpremultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = pixel[3] as u32;
        let mut b = pixel[2] as u32;
        let mut g = pixel[1] as u32;
        let mut r = pixel[0] as u32;

        if a > 0 {
            r = r * 255 / a;
            g = g * 255 / a;
            b = b * 255 / a;
        }

        pixel[3] = a as u8;
        pixel[2] = r as u8;
        pixel[1] = g as u8;
        pixel[0] = b as u8;
    }
}

fn main() {

    let mut arena = Arena::new();
    let mut r = rasterizer::Rasterizer::new(&mut arena, 400, 400);

    let mut p = PathBuilder::new(&mut r);
    p.move_to(50., 50.);
    p.line_to(100., 70.);
    p.line_to(110., 150.);
    p.line_to(40., 180.);
    p.close();

    p.move_to(100., 10.);
    p.quad_to(150., 40., 200., 10.);
    p.quad_to(120., 100., 80., 200.);
    p.quad_to(150., 180., 200., 200.);
    p.close();


    r.rasterize();
    let file = File::create("out.png").unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 400, 400); // Width is 2 pixels and height is 1.
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let buf = r.buf[..].as_mut_ptr();
    let mut buf8 = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, r.buf.len() * 4) };
    unpremultiply(&mut buf8);
    writer.write_image_data(buf8).unwrap();
}
