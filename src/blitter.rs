use sw_composite::*;

pub trait Blitter {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32);
}

pub struct MaskSuperBlitter {
    width: i32,
    height: i32,
    pub buf: Vec<u8>,
}

const SHIFT: i32 = 2;
const MASK: i32 = (SCALE - 1);
const SCALE: i32 = (1 << SHIFT);
const SUPER_Mask: i32 = ((1 << SHIFT) - 1);

fn coverage_to_alpha(mut aa: i32) -> u8
{
    aa <<= 8 - 2 * SHIFT;
    aa -= aa >> (8 - SHIFT - 1);
    return aa as u8;
}


impl MaskSuperBlitter {
    pub fn new(width: i32, height: i32) -> MaskSuperBlitter {
        MaskSuperBlitter { width, height, buf: vec![0; (width * height) as usize] }
    }
}

impl Blitter for MaskSuperBlitter {
    fn blit_span(&mut self, y: i32, x1: i32, x2: i32) {
        let max: u8 = ((1 << (8 - SHIFT)) - (((y & MASK) + 1) >> SHIFT)) as u8;
        let mut b: *mut u8 = &mut self.buf[(y / 4 * self.width + (x1 >> SHIFT)) as usize];

        let mut fb = x1 & SUPER_Mask;
        let fe = x2 & SUPER_Mask;
        let mut n = (x2 >> SHIFT) - (x1 >> SHIFT) - 1;

        // invert the alpha on the left side
        if n < 0 {
            unsafe { *b += coverage_to_alpha(fe - fb) };
        } else {
            fb = (1 << SHIFT) - fb;
            unsafe { *b += coverage_to_alpha(fb) };
            unsafe { b = b.offset(1); };
            while n != 0 {
                unsafe { *b += max };
                unsafe { b = b.offset(1) };

                n -= 1;
            }
            unsafe { *b += coverage_to_alpha(fe) };
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
            self.dest[(dest_row + i) as usize] =
                over_in(self.color,
                        self.dest[(dest_row + i) as usize],
                        self.mask[(mask_row + i) as usize] as u32);
        }
    }
}
