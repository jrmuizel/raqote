mod rasterizer;
mod types;
use typed_arena::Arena;
use types::Point;
use std::fs::*;
use std::io::BufWriter;

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
    r.add_edge(Point{x: 50., y: 50.}, Point{x: 100., y: 70.}, false, Point{x: 0., y: 0.});
    r.add_edge(Point{x: 100., y: 70.}, Point{x: 110., y: 150.}, false, Point{x: 0., y: 0.});
    r.add_edge(Point{x: 110., y: 150.}, Point{x: 40., y: 180.}, false, Point{x: 0., y: 0.});
    r.add_edge(Point{x: 40., y: 180.}, Point{x: 50., y: 50.}, false, Point{x: 0., y: 0.});
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
