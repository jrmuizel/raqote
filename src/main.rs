mod rasterizer;
mod types;
mod geom;
mod blitter;
mod draw_target;
mod stroke;

use typed_arena::Arena;
use types::Point;
use std::fs::*;
use std::io::BufWriter;

mod path_builder;
use path_builder::PathBuilder;

use png::HasParameters;

use crate::draw_target::{DrawTarget, Source, SolidSource};

use sw_composite::*;
use crate::blitter::MaskSuperBlitter;


fn unpremultiply(data: &mut [u8]) {
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

    let mut dt = DrawTarget::new(400, 400);
    let mut pb = PathBuilder::new();
    pb.move_to(50., 50.);
    pb.line_to(100., 70.);
    pb.line_to(110., 150.);
    pb.line_to(40., 180.);
    pb.close();

    /*
    dt.move_to(100., 10.);
    dt.quad_to(150., 40., 200., 10.);
    dt.quad_to(120., 100., 80., 200.);
    dt.quad_to(150., 180., 200., 200.);
    dt.close();
    */

    pb.move_to(100., 10.);
    pb.cubic_to(150., 40., 175., 0., 200., 10.);
    pb.quad_to(120., 100., 80., 200.);
    pb.quad_to(150., 180., 200., 200.);
    pb.close();

    let path = pb.finish();

    let decoder = png::Decoder::new(File::open("photo.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    println!("{:?}", info.color_type);

    let mut image : Vec<u32> = Vec::new();
    for i in buf.chunks(3) {
        image.push(0xff << 24 | ((i[0]as u32) << 16) | ((i[1] as u32) << 8) | (i[2] as u32))
    }
    let bitmap = Bitmap { width: info.width as i32, height: info.height as i32, data: image};

    //dt.fill(Source::Solid(SolidSource{r: 0xff, g: 0xff, b: 0, a: 0xff}));
    //dt.fill(Source::Bitmap(bitmap, euclid::Transform2D::create_scale(2., 2.)));

    dt.fill(&path, Source::Gradient(Gradient { stops: vec![GradientStop{position: 0.2, color: 0xff00ff00},
                                                    GradientStop{position: 0.8, color: 0xffffffff},
                                                    GradientStop{position: 1., color: 0xffff00ff}]},
            euclid::Transform2D::create_translation(-50., -50.)));

    let file = File::create("out.png").unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 400, 400); // Width is 2 pixels and height is 1.
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let buf = dt.buf[..].as_mut_ptr();
    let mut buf8 = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, dt.buf.len() * 4) };
    unpremultiply(&mut buf8);
    writer.write_image_data(buf8).unwrap();
}
