use sw_composite::*;

use crate::Transform;

pub trait Blitter {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32);
}

pub struct MaskSuperBlitter {
    width: i32,
    height: i32,
    pub buf: Vec<u8>,
}

const SHIFT: i32 = 2;
const SCALE: i32 = (1 << SHIFT);
const MASK: i32 = (SCALE - 1);
const SUPER_MASK: i32 = ((1 << SHIFT) - 1);

fn coverage_to_partial_alpha(mut aa: i32) -> u8 {
    aa <<= 8 - 2 * SHIFT;
    return aa as u8;
}

impl MaskSuperBlitter {
    pub fn new(width: i32, height: i32) -> MaskSuperBlitter {
        MaskSuperBlitter {
            width,
            height,
            // we can end up writing one byte past the end of the buffer so allocate that
            // padding to avoid needing to do an extra check
            buf: vec![0; (width * height) as usize + 1],
        }
    }
}

// Perform this tricky subtract, to avoid overflowing to 256. Our caller should
// only ever call us with at most enough to hit 256 (never larger), so it is
// enough to just subtract the high-bit. Actually clamping with a branch would
// be slower (e.g. if (tmp > 255) tmp = 255;)
fn saturated_add(a: u8, b: u8) -> u8 {
    let tmp = a as u32 + b as u32;
    let result = (tmp - (tmp >> 8));
    result as u8
}

impl Blitter for MaskSuperBlitter {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32) {
        let max: u8 = ((1 << (8 - SHIFT)) - (((y & MASK) + 1) >> SHIFT)) as u8;
        let mut b: *mut u8 = &mut self.buf[(y / 4 * self.width + (x1 >> SHIFT)) as usize];

        let mut fb = x1 & SUPER_MASK;
        let fe = x2 & SUPER_MASK;
        let mut n = (x2 >> SHIFT) - (x1 >> SHIFT) - 1;

        // invert the alpha on the left side
        if n < 0 {
            unsafe { *b = saturated_add(*b, coverage_to_partial_alpha(fe - fb)) };
        } else {
            fb = (1 << SHIFT) - fb;
            unsafe { *b = saturated_add(*b, coverage_to_partial_alpha(fb)) };
            unsafe {
                b = b.offset(1);
            };
            while n != 0 {
                unsafe { *b += max };
                unsafe { b = b.offset(1) };

                n -= 1;
            }
            unsafe { *b = saturated_add(*b, coverage_to_partial_alpha(fe)) };
        }
    }
}

pub trait Shader {
    fn shade_span(&self, x: i32, y: i32, dest: &mut [u32], count: usize);
}

pub struct SolidShader {
    pub color: u32,
}

impl Shader for SolidShader {
    fn shade_span(&self, x: i32, y: i32, dest: &mut [u32], count: usize) {
        for i in 0..count {
            dest[i] = self.color;
        }
    }
}

fn transform_to_fixed(transform: &Transform) -> MatrixFixedPoint {
    MatrixFixedPoint {
        xx: float_to_fixed(transform.m11),
        xy: float_to_fixed(transform.m21),
        yx: float_to_fixed(transform.m12),
        yy: float_to_fixed(transform.m22),
        x0: float_to_fixed(transform.m31),
        y0: float_to_fixed(transform.m32),
    }
}

pub struct ImageShader<'a> {
    image: &'a Image,
    xfm: MatrixFixedPoint,
}

impl<'a> ImageShader<'a> {
    pub fn new(image: &'a Image, transform: &Transform) -> ImageShader<'a> {
        ImageShader {
            image,
            xfm: transform_to_fixed(transform),
        }
    }
}

impl<'a> Shader for ImageShader<'a> {
    fn shade_span(&self, mut x: i32, y: i32, dest: &mut [u32], count: usize) {
        for i in 0..count {
            let p = self.xfm.transform(x as u16, y as u16);
            dest[i] = fetch_bilinear(self.image, p.x, p.y);
            x += 1;
        }
    }
}

pub struct RadialGradientShader {
    gradient: Box<GradientSource>,
}

impl RadialGradientShader {
    pub fn new(gradient: &Gradient, transform: &Transform) -> RadialGradientShader {
        RadialGradientShader {
            gradient: gradient.make_source(&transform_to_fixed(transform)),
        }
    }
}

impl Shader for RadialGradientShader {
    fn shade_span(&self, mut x: i32, y: i32, dest: &mut [u32], count: usize) {
        for i in 0..count {
            dest[i] = self.gradient.radial_gradient_eval(x as u16, y as u16);
            x += 1;
        }
    }
}

pub struct LinearGradientShader {
    gradient: Box<GradientSource>,
}

impl LinearGradientShader {
    pub fn new(gradient: &Gradient, transform: &Transform) -> LinearGradientShader {
        LinearGradientShader {
            gradient: gradient.make_source(&transform_to_fixed(transform)),
        }
    }
}

impl Shader for LinearGradientShader {
    fn shade_span(&self, mut x: i32, y: i32, dest: &mut [u32], count: usize) {
        for i in 0..count {
            dest[i] = self.gradient.linear_gradient_eval(x as u16, y as u16);
            x += 1;
        }
    }
}

pub struct ShaderBlitter<'a> {
    pub shader: &'a Shader,
    pub tmp: Vec<u32>,
    pub dest: &'a mut [u32],
    pub dest_stride: i32,
    pub mask: &'a [u8],
    pub mask_stride: i32,
}

impl<'a> Blitter for ShaderBlitter<'a> {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32) {
        let dest_row = y * self.dest_stride;
        let mask_row = y * self.mask_stride;
        let count = (x2 - x1) as usize;
        self.shader.shade_span(x1, y, &mut self.tmp[..], count);
        for i in 0..count {
            self.dest[(dest_row + x1) as usize + i] = over_in(
                self.tmp[i],
                self.dest[(dest_row + x1) as usize + i],
                self.mask[(mask_row + x1) as usize + i] as u32,
            );
        }
    }
}

pub struct ShaderClipBlitter<'a> {
    pub shader: &'a Shader,
    pub tmp: Vec<u32>,
    pub dest: &'a mut [u32],
    pub dest_stride: i32,
    pub mask: &'a [u8],
    pub mask_stride: i32,
    pub clip: &'a [u8],
    pub clip_stride: i32,
}

impl<'a> Blitter for ShaderClipBlitter<'a> {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32) {
        let dest_row = y * self.dest_stride;
        let mask_row = y * self.mask_stride;
        let clip_row = y * self.clip_stride;
        let count = (x2 - x1) as usize;
        self.shader.shade_span(x1, y, &mut self.tmp[..], count);
        for i in 0..count {
            self.dest[(dest_row + x1) as usize + i] = over_in_in(
                self.tmp[i],
                self.dest[(dest_row + x1) as usize + i],
                self.mask[(mask_row + x1) as usize + i] as u32,
                self.clip[(clip_row + x1) as usize + i] as u32,
            );
        }
    }
}

pub struct ShaderClipBlendBlitter<'a> {
    pub shader: &'a Shader,
    pub tmp: Vec<u32>,
    pub dest: &'a mut [u32],
    pub dest_stride: i32,
    pub mask: &'a [u8],
    pub mask_stride: i32,
    pub clip: &'a [u8],
    pub clip_stride: i32,
}

impl<'a> Blitter for ShaderClipBlendBlitter<'a> {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32) {
        let dest_row = y * self.dest_stride;
        let mask_row = y * self.mask_stride;
        let clip_row = y * self.clip_stride;
        let count = (x2 - x1) as usize;
        self.shader.shade_span(x1, y, &mut self.tmp[..], count);
        for i in 0..count {
            self.dest[(dest_row + x1) as usize + i] = over_in_in(
                self.tmp[i],
                self.dest[(dest_row + x1) as usize + i],
                self.mask[(mask_row + x1) as usize + i] as u32,
                self.clip[(clip_row + x1) as usize + i] as u32,
            );
        }
    }
}

pub struct SolidBlitter<'a> {
    color: u32,
    mask: &'a [u8],
    dest: &'a mut [u32],
    dest_stride: i32,
    mask_stride: i32,
}

impl<'a> Blitter for SolidBlitter<'a> {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32) {
        let dest_row = y * self.dest_stride;
        let mask_row = y * self.mask_stride;
        for i in x1..x2 {
            self.dest[(dest_row + i) as usize] = over_in(
                self.color,
                self.dest[(dest_row + i) as usize],
                self.mask[(mask_row + i) as usize] as u32,
            );
        }
    }
}
