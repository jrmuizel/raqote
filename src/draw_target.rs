use crate::rasterizer::Rasterizer;

use crate::blitter::*;
use sw_composite::*;

use crate::dash::*;
use crate::geom::*;
use crate::path_builder::*;

pub use crate::path_builder::Winding;
use lyon_geom::cubic_to_quadratic::cubic_to_quadratics;
use lyon_geom::CubicBezierSegment;

#[cfg(feature = "text")]
mod fk {
    pub use font_kit::canvas::{Canvas, Format, RasterizationOptions};
    pub use font_kit::font::Font;
    pub use font_kit::hinting::HintingOptions;
    pub use font_kit::loader::FontTransform;
}

use std::fs::*;
use std::io::BufWriter;

use crate::stroke::*;
use crate::{IntRect, IntPoint, Point, Transform, Vector};

use euclid::vec2;

#[derive(Clone)]
pub struct Mask {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

/// A premultiplied colored. i.e. r,b,g <= a
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SolidSource {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BlendMode {
    Dst,
    Src,
    Clear,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcOut,
    DstOut,
    SrcAtop,
    DstAtop,
    Xor,
    Add,

    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    Hue,
    Saturation,
    Color,
    Luminosity
}

fn blend_proc(mode: BlendMode) -> fn(u32, u32) -> u32 {
    match mode {
        BlendMode::Dst => dst,
        BlendMode::Src => src,
        BlendMode::Clear => clear,
        BlendMode::SrcOver => src_over,
        BlendMode::DstOver => dst_over,
        BlendMode::SrcIn => src_in,
        BlendMode::DstIn => dst_in,
        BlendMode::SrcOut => src_out,
        BlendMode::DstOut => dst_out,
        BlendMode::SrcAtop => src_atop,
        BlendMode::DstAtop => dst_atop,
        BlendMode::Xor => xor,
        BlendMode::Add => add,
        BlendMode::Screen => screen,
        BlendMode::Overlay => overlay,
        BlendMode::Darken => darken,
        BlendMode::Lighten => lighten,
        BlendMode::ColorDodge => colordodge,
        BlendMode::ColorBurn => colorburn,
        BlendMode::HardLight => hardlight,
        BlendMode::SoftLight => softlight,
        BlendMode::Difference => difference,
        BlendMode::Exclusion => exclusion,
        BlendMode::Multiply => multiply,
        BlendMode::Hue => hue,
        BlendMode::Saturation => saturation,
        BlendMode::Color => color,
        BlendMode::Luminosity => luminosity,
    }
}

#[derive(Copy, Clone)]
pub enum ExtendMode {
    Pad,
    Repeat
}

#[derive(Copy, Clone, PartialEq)]
pub enum FilterMode {
    Bilinear,
    Nearest
}

/// LinearGradients have an implicit start point at 0,0 and an end point at 256,0. The transform
/// parameter can be used to adjust them to the desired location.
/// RadialGradients have an implict center at 0,0 and a radius of 128.
///
/// These locations are an artifact of the blitter implementation and will probably change in the
/// future to become more ergonomic.
#[derive(Clone)]
pub enum Source<'a> {
    Solid(SolidSource),
    Image(Image<'a>, ExtendMode, FilterMode, Transform),
    RadialGradient(Gradient, Spread, Transform),
    TwoCircleRadialGradient(Gradient, Spread, Point, f32, Point, f32, Transform),
    LinearGradient(Gradient, Spread, Transform),
}

impl<'a> Source<'a> {
    /// Creates a new linear gradient source where the start point corresponds to the gradient
    /// stop at position = 0 and the end point corresponds to the graident stop at position = 1.
    pub fn new_linear_gradient(gradient: Gradient, start: Point, end: Point, spread: Spread) -> Source<'a> {
        let gradient_vector = Vector::new(end.x - start.x, end.y - start.y);
        // Get length of desired gradient vector
        let length = gradient_vector.length();
        let gradient_vector = gradient_vector.normalize();

        let sin = gradient_vector.y;
        let cos = gradient_vector.x;
        // Build up a rotation matrix from our vector
        let mat = Transform::row_major(cos, -sin, sin, cos, 0., 0.);

        // Adjust for the start point
        let mat = mat.pre_translate(vec2(-start.x, -start.y));

        // Scale gradient to desired length
        let mat = mat.post_scale(1./length, 1./length);

        Source::LinearGradient(gradient, spread, mat)
    }

    /// Creates a new radial gradient that is centered at the given point and has the given radius.
    pub fn new_radial_gradient(gradient: Gradient, center: Point, radius: f32, spread: Spread) -> Source<'a> {
        // Scale gradient to desired radius
        let scale = Transform::create_scale(radius, radius);
        // Transform gradient to center of gradient
        let translate = Transform::create_translation(center.x, center.y);
        // Compute final transform
        let transform = translate.pre_transform(&scale).inverse().unwrap();

        Source::RadialGradient(gradient, spread, transform)
    }

    /// Creates a new radial gradient that is centered at the given point and has the given radius.
    pub fn new_two_circle_radial_gradient(gradient: Gradient, center1: Point, radius1: f32,  center2: Point, radius2: f32, spread: Spread) -> Source<'a> {
        let transform = Transform::identity();
        Source::TwoCircleRadialGradient(gradient, spread, center1, radius1, center2, radius2, transform)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AntialiasMode {
    None,
    Gray,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct DrawOptions {
    pub blend_mode: BlendMode,
    pub alpha: f32,
    pub antialias: AntialiasMode,
}

impl DrawOptions {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for DrawOptions {
    fn default() -> Self {
        DrawOptions {
            blend_mode: BlendMode::SrcOver,
            alpha: 1.,
            antialias: AntialiasMode::Gray,
        }
    }
}

#[derive(Clone)]
struct Clip {
    rect: IntRect,
    mask: Option<Vec<u8>>,
}

#[derive(Clone)]
struct Layer {
    buf: Vec<u32>,
    opacity: f32,
    rect: IntRect,
}

fn scaled_tolerance(x: f32, trans: &Transform) -> f32 {
    // The absolute value of the determinant is the area parallelogram
    // Take the sqrt of the area to losily convert to one dimension
    x / trans.determinant().abs().sqrt()
}



/// The main type used for drawing
pub struct DrawTarget {
    width: i32,
    height: i32,
    rasterizer: Rasterizer,
    current_point: Point,
    first_point: Option<Point>,
    buf: Vec<u32>,
    clip_stack: Vec<Clip>,
    layer_stack: Vec<Layer>,
    transform: Transform,
}

impl DrawTarget {
    pub fn new(width: i32, height: i32) -> DrawTarget {
        DrawTarget {
            width,
            height,
            current_point: Point::new(0., 0.),
            first_point: None,
            rasterizer: Rasterizer::new(width, height),
            buf: vec![0; (width * height) as usize],
            clip_stack: Vec::new(),
            layer_stack: Vec::new(),
            transform: Transform::identity(),
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    /// sets a transform that will be applied to all drawing operations
    pub fn set_transform(&mut self, transform: &Transform) {
        self.transform = *transform;
    }

    /// gets the current transform
    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn move_to(&mut self, pt: Point) {
        self.current_point = pt;
        self.first_point = Some(pt);
    }

    fn line_to(&mut self, pt: Point) {
        self.rasterizer
            .add_edge(self.current_point, pt, false, Point::new(0., 0.));
        self.current_point = pt;
    }

    fn quad_to(&mut self, cpt: Point, pt: Point) {
        let curve = [self.current_point, cpt, pt];
        self.current_point = curve[2];
        self.add_quad(curve);
    }

    fn add_quad(&mut self, mut curve: [Point; 3]) {
        let a = curve[0].y;
        let b = curve[1].y;
        let c = curve[2].y;
        if is_not_monotonic(a, b, c) {
            let mut t_value = 0.;
            if valid_unit_divide(a - b, a - b - b + c, &mut t_value) {
                let mut dst = [Point::new(0., 0.); 5];
                chop_quad_at(&curve, &mut dst, t_value);
                flatten_double_quad_extrema(&mut dst);
                self.rasterizer.add_edge(dst[0], dst[2], true, dst[1]);
                self.rasterizer.add_edge(dst[2], dst[4], true, dst[3]);
                return;
            }
            // if we get here, we need to force dst to be monotonic, even though
            // we couldn't compute a unit_divide value (probably underflow).
            let b = if abs(a - b) < abs(b - c) { a } else { c };
            curve[1].y = b;
        }
        self.rasterizer.add_edge(curve[0], curve[2], true, curve[1]);
    }

    fn cubic_to(&mut self, cpt1: Point, cpt2: Point, pt: Point) {
        let c = CubicBezierSegment {
            from: self.current_point,
            ctrl1: cpt1,
            ctrl2: cpt2,
            to: pt,
        };
        cubic_to_quadratics(&c, 0.01, &mut |q| {
            let curve = [q.from, q.ctrl, q.to];
            self.add_quad(curve);
        });
        self.current_point = pt;
    }

    fn close(&mut self) {
        if let Some(first_point) = self.first_point {
            self.rasterizer.add_edge(
                self.current_point,
                first_point,
                false,
                Point::new(0., 0.),
            );
        }
        self.first_point = None;
    }

    fn apply_path(&mut self, path: &Path) {
        for op in &path.ops {
            match *op {
                PathOp::MoveTo(pt) => {
                    self.close();
                    self.move_to(self.transform.transform_point(pt));
                },
                PathOp::LineTo(pt) => self.line_to(self.transform.transform_point(pt)),
                PathOp::QuadTo(cpt, pt) => self.quad_to(
                    self.transform.transform_point(cpt),
                    self.transform.transform_point(pt),
                ),
                PathOp::CubicTo(cpt1, cpt2, pt) => self.cubic_to(
                    self.transform.transform_point(cpt1),
                    self.transform.transform_point(cpt2),
                    self.transform.transform_point(pt),
                ),
                PathOp::Close => self.close(),
            }
        }
        // make sure the path is closed
        self.close();
        // XXX: we'd like for this function to return the bounds of the path
    }

    pub fn push_clip_rect(&mut self, rect: IntRect) {
        // intersect with current clip
        let clip = match self.clip_stack.last() {
            Some(Clip {
                     rect: current_clip,
                     mask: _,
                 }) => Clip {
                rect: current_clip.intersection(&rect),
                mask: None,
            },
            _ => Clip {
                rect: rect,
                mask: None,
            },
        };
        self.clip_stack.push(clip);
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    pub fn push_clip(&mut self, path: &Path) {
        self.apply_path(path);

        // XXX: restrict to clipped area
        let mut blitter = MaskSuperBlitter::new(0, 0, self.width, self.height);
        self.rasterizer.rasterize(&mut blitter, path.winding);

        if let Some(last) = self.clip_stack.last() {
            // combine with previous mask
            if let Some(last_mask) = &last.mask {
                for i in 0..((self.width * self.height) as usize) {
                    blitter.buf[i] = muldiv255(blitter.buf[i] as u32, last_mask[i] as u32) as u8
                }
            }
        }

        let current_bounds = self.clip_bounds();
        //XXX: handle interleaving of clip rect/masks better
        self.clip_stack.push(Clip {
            rect: current_bounds,
            mask: Some(blitter.buf),
        });
        self.rasterizer.reset();
    }

    fn clip_bounds(&self) -> IntRect {
        self.clip_stack.last().map(|c| c.rect).unwrap_or(IntRect::new(
            euclid::Point2D::new(0, 0),
            euclid::Point2D::new(self.width, self.height),
        ))
    }

    /// Pushes a new layer as the drawing target. This is used for implementing
    /// group opacity effects.
    pub fn push_layer(&mut self, opacity: f32) {
        let rect = self.clip_bounds();
        self.layer_stack.push(Layer {
            rect,
            buf: vec![0; (rect.size().width * rect.size().height) as usize],
            opacity,
        });
    }

    /// Draws the most recently pushed layer to the drawing target with
    /// the pushed opacity applied.
    pub fn pop_layer(&mut self) {
        let layer = self.layer_stack.pop().unwrap();
        let opacity = (layer.opacity * 255. + 0.5) as u8;
        // Allocating an entire mask just for the opacity is needlessly bad.
        // We should be able to fix it once the blitters work better.
        let mask = vec![opacity; (self.width * self.height) as usize];
        let size = layer.rect.size();
        let ctm = self.transform;
        self.transform = Transform::identity();
        let image = Source::Image(Image {
            width: size.width,
            height: size.height,
            data: &layer.buf
        },
                                  ExtendMode::Pad,
                                  FilterMode::Nearest,
                                  Transform::create_translation(-layer.rect.min.x as f32,
                                                                -layer.rect.min.y as f32));
        self.composite(&image, &mask, intrect(0, 0, self.width, self.height), layer.rect, BlendMode::SrcOver, 1.);
        self.transform = ctm;
    }

    /// Draws an image at (x, y) with the size (width, height). This will rescale the image to the
    /// destination size.
    pub fn draw_image_with_size_at(&mut self, width: f32, height: f32, x: f32, y: f32, image: &Image, options: &DrawOptions) {
        let mut pb = PathBuilder::new();
        pb.rect(x, y, width, height);
        let source = Source::Image(*image,
                                   ExtendMode::Pad,
                                   FilterMode::Bilinear,
                                   Transform::create_translation(-x, -y).post_scale(image.width as f32 / width, image.height as f32 / height));

        self.fill(&pb.finish(), &source, options);
    }

    /// Draws an image at x, y
    pub fn draw_image_at(&mut self, x: f32, y: f32, image: &Image, options: &DrawOptions) {
        self.draw_image_with_size_at(image.width as f32, image.height as f32, x, y, image, options);
    }

    /// Draws `src` through an untransformed `mask` positioned at `x`, `y` in device space
    pub fn mask(&mut self, src: &Source, x: i32, y: i32, mask: &Mask) {
        self.composite(src, &mask.data, intrect(x, y, mask.width, mask.height), intrect(x, y, mask.width, mask.height), BlendMode::SrcOver, 1.);
    }

    /// Strokes `path` with `style` and fills the result with `src`
    pub fn stroke(&mut self, path: &Path, src: &Source, style: &StrokeStyle, options: &DrawOptions) {
        let tolerance = 0.1;

        // Since we're flattening in userspace, we need to compensate for the transform otherwise
        // we'll flatten too much or not enough depending on the scale. We approximate doing this
        // correctly by scaling the tolerance value using the same mechanism as Fitz. This
        // approximation will fail if the scale between axes is drastically different. An
        // alternative would be to use transform specific flattening but I haven't seen that done
        // anywhere.
        let tolerance = scaled_tolerance(tolerance, &self.transform);
        let mut path = path.flatten(tolerance);

        if !style.dash_array.is_empty() {
            path = dash_path(&path, &style.dash_array, style.dash_offset);
        }
        let stroked = stroke_to_path(&path, style);
        self.fill(&stroked, src, options);
    }

    /// Fills `path` with `src`
    pub fn fill(&mut self, path: &Path, src: &Source, options: &DrawOptions) {
        self.apply_path(path);
        let bounds = self.rasterizer.get_bounds();
        match options.antialias {
            AntialiasMode::None => {
                let mut blitter = MaskBlitter::new(bounds.min.x, bounds.min.y, bounds.size().width, bounds.size().height);
                self.rasterizer.rasterize(&mut blitter, path.winding);
                self.composite(
                    src,
                    &blitter.buf,
                    bounds,
                    bounds,
                    options.blend_mode,
                    options.alpha,
                );
            }
            AntialiasMode::Gray => {
                let mut blitter = MaskSuperBlitter::new(bounds.min.x, bounds.min.y, bounds.size().width, bounds.size().height);
                self.rasterizer.rasterize(&mut blitter, path.winding);
                self.composite(
                    src,
                    &blitter.buf,
                    bounds,
                    bounds,
                    options.blend_mode,
                    options.alpha,
                );
            }
        }
        self.rasterizer.reset();
    }

    /// Fills the current clip with the solid color `solid`
    pub fn clear(&mut self, solid: SolidSource) {
        let mut pb = PathBuilder::new();
        let ctm = self.transform;
        self.transform = Transform::identity();
        pb.rect(0., 0., self.width as f32, self.height as f32);
        self.fill(
            &pb.finish(),
            &Source::Solid(solid),
            &DrawOptions {
                blend_mode: BlendMode::Src,
                alpha: 1.,
                antialias: AntialiasMode::Gray,
            },
        );
        self.transform = ctm;
    }

    #[cfg(feature = "text")]
    pub fn draw_text(
        &mut self,
        font: &fk::Font,
        point_size: f32,
        text: &str,
        mut start: Point,
        src: &Source,
        options: &DrawOptions,
    ) {
        let mut ids = Vec::new();
        let mut positions = Vec::new();
        for c in text.chars() {
            let id = font.glyph_for_char(c).unwrap();
            ids.push(id);
            positions.push(start);
            start += font.advance(id).unwrap() / 96.;
        }
        self.draw_glyphs(font, point_size, &ids, &positions, src, options);
    }

    #[cfg(feature = "text")]
    pub fn draw_glyphs(
        &mut self,
        font: &fk::Font,
        point_size: f32,
        ids: &[u32],
        positions: &[Point],
        src: &Source,
        options: &DrawOptions,
    ) {
        let mut combined_bounds = euclid::Rect::zero();
        for (id, position) in ids.iter().zip(positions.iter()) {
            let bounds = font.raster_bounds(
                *id,
                point_size,
                &fk::FontTransform::identity(),
                position,
                fk::HintingOptions::None,
                fk::RasterizationOptions::GrayscaleAa,
            );
            combined_bounds = match bounds {
                Ok(bounds) => {
                    combined_bounds.union(&bounds)
                }
                _ => panic!(),
            }
        }

        /*let mut canvas = Canvas::new(&euclid::Size2D::new(combined_bounds.size.width as u32,
        combined_bounds.size.height as u32), Format::A8);*/
        let mut canvas = fk::Canvas::new(
            &euclid::Size2D::new(self.width as u32, self.height as u32),
            fk::Format::A8,
        );
        for (id, position) in ids.iter().zip(positions.iter()) {
            font.rasterize_glyph(
                &mut canvas,
                *id,
                point_size,
                &fk::FontTransform::new(self.transform.m11, self.transform.m12, self.transform.m21, self.transform.m22),
                &(self.transform.transform_point(*position)),
                fk::HintingOptions::None,
                fk::RasterizationOptions::GrayscaleAa,
            ).unwrap();
        }

        self.composite(
            src,
            &canvas.pixels,
            intrect(0, 0, self.width, self.height),
            intrect(0, 0, canvas.size.width as i32, canvas.size.height as i32),
            options.blend_mode,
            1.,
        );
    }




    fn choose_blitter<'a, 'b, 'c>(clip_stack: &'a Vec<Clip>, blitter_storage: &'b mut ShaderBlitterStorage<'a>, shader: &'a dyn Shader, blend: BlendMode, dest: &'a mut [u32], dest_bounds: IntRect, width: i32) -> &'b mut dyn Blitter {
        let blitter: &mut dyn Blitter;

        if blend == BlendMode::SrcOver {
            match clip_stack.last() {
                Some(Clip {
                         rect: _,
                         mask: Some(clip),
                     }) => {
                    let scb = ShaderClipBlitter {
                        x: dest_bounds.min.x,
                        y: dest_bounds.min.y,
                        shader: shader,
                        tmp: vec![0; width as usize],
                        dest,
                        dest_stride: dest_bounds.size().width,
                        clip,
                        clip_stride: width,
                    };
                    *blitter_storage = ShaderBlitterStorage::ShaderClipBlitter(scb);
                    blitter = match blitter_storage { ShaderBlitterStorage::ShaderClipBlitter(s) => s, _ => panic!() };
                }
                _ => {
                    let sb = ShaderBlitter {
                        x: dest_bounds.min.x,
                        y: dest_bounds.min.y,
                        shader: &*shader,
                        tmp: vec![0; width as usize],
                        dest,
                        dest_stride: dest_bounds.size().width,
                    };
                    *blitter_storage = ShaderBlitterStorage::ShaderBlitter(sb);
                    blitter = match blitter_storage { ShaderBlitterStorage::ShaderBlitter(s) => s, _ => panic!() };
                }
            }
        } else {

            let blend_fn = blend_proc(blend);
            match clip_stack.last() {
                Some(Clip {
                         rect: _,
                         mask: Some(clip),
                     }) => {
                    let scb_blend = ShaderClipBlendBlitter {
                        x: dest_bounds.min.x,
                        y: dest_bounds.min.y,
                        shader: shader,
                        tmp: vec![0; width as usize],
                        dest,
                        dest_stride: dest_bounds.size().width,
                        clip,
                        clip_stride: width,
                        blend_fn
                    };

                    *blitter_storage = ShaderBlitterStorage::ShaderClipBlendBlitter(scb_blend);
                    blitter = match blitter_storage {
                        ShaderBlitterStorage::ShaderClipBlendBlitter(s) => s,
                        _ => panic!()
                    };
                }
                _ => {
                    let sb_blend = ShaderBlendBlitter {
                        x: dest_bounds.min.x,
                        y: dest_bounds.min.y,
                        shader: &*shader,
                        tmp: vec![0; width as usize],
                        dest,
                        dest_stride: dest_bounds.size().width,
                        blend_fn
                    };
                    *blitter_storage = ShaderBlitterStorage::ShaderBlendBlitter(sb_blend);
                    blitter = match blitter_storage {
                        ShaderBlitterStorage::ShaderBlendBlitter(s) => s,
                        _ => panic!()
                    };
                }
            }
        }
        blitter
    }

    fn composite(&mut self, src: &Source, mask: &[u8], mask_rect: IntRect, mut rect: IntRect, blend: BlendMode, alpha: f32) {
        let ti = self.transform.inverse();
        let ti = if let Some(ti) = ti {
            ti
        } else {
            // the transform is not invertible so we have nothing to draw
            return;
        };

        let clip_bounds = self.clip_bounds();

        let (dest, dest_bounds) = match self.layer_stack.last_mut() {
            Some(layer) => (&mut layer.buf[..], layer.rect),
            None => (&mut self.buf[..], intrect(0, 0, self.width, self.height))
        };

        rect = rect
            .intersection(&clip_bounds)
            .intersection(&dest_bounds)
            .intersection(&mask_rect);
        if rect.is_negative() {
            return;
        }

        let mut shader_storage: ShaderStorage = ShaderStorage::None;
        let shader = choose_shader(&ti, src, alpha, &mut shader_storage);

        let mut blitter_storage: ShaderBlitterStorage = ShaderBlitterStorage::None;
        let blitter = DrawTarget::choose_blitter(&self.clip_stack, &mut blitter_storage, shader, blend, dest, dest_bounds, self.width);

        for y in rect.min.y..rect.max.y {
            let mask_row = (y - mask_rect.min.y) * mask_rect.size().width;
            let mask_start = (mask_row + rect.min.x - mask_rect.min.x) as usize;
            let mask_end = (mask_row + rect.max.x - mask_rect.min.x) as usize;
            blitter.blit_span(y, rect.min.x, rect.max.x, &mask[mask_start..mask_end]);
        }
    }

    /// Draws `src_rect` of `src` at `dst`. The current transform and clip are ignored
    pub fn copy_surface(&mut self, src: &DrawTarget, src_rect: IntRect, dst: IntPoint) {
        let dst_rect = intrect(0, 0, self.width, self.height);
        let src_rect = dst_rect
            .intersection(&src_rect.translate(dst.to_vector())).translate(-dst.to_vector());

        // clamp requires Float so open code it
        let dst = IntPoint::new(dst.x.max(dst_rect.min.x).min(dst_rect.max.x),
                                dst.y.max(dst_rect.min.y).min(dst_rect.max.y));

        if src_rect.is_negative() {
            return;
        }

        for y in src_rect.min.y..src_rect.max.y {
            let dst_row_start = (dst.x + (dst.y + y - src_rect.min.y) * self.width) as usize;
            let dst_row_end = dst_row_start + src_rect.size().width as usize;
            let src_row_start = (src_rect.min.x + y * src.width) as usize;
            let src_row_end = src_row_start + src_rect.size().width as usize;
            self.buf[dst_row_start..dst_row_end].copy_from_slice(&src.buf[src_row_start..src_row_end]);
        }
    }

    /// Returns a reference to the underlying pixel data
    pub fn get_data(&self) -> &[u32] {
        &self.buf
    }

    /// Returns a mut reference to the underlying pixel data as ARGB with a representation
    /// like: (A << 24) | (R << 16) | (G << 8) | B
    pub fn get_data_mut(&mut self) -> &mut [u32] {
        &mut self.buf
    }

    /// Returns a mut reference to the underlying pixel data as individual bytes with the order BGRA
    /// on little endian.
    pub fn get_data_u8_mut(&mut self) -> &mut [u8] {
        let p = self.buf[..].as_mut_ptr();
        let len = self.buf[..].len();
        // we want to return an [u8] slice instead of a [u32] slice. This is a safe thing to
        // do because requirements of a [u32] slice are stricter.
        unsafe { std::slice::from_raw_parts_mut(p as *mut u8, len * std::mem::size_of::<u32>()) }
    }

    /// Take ownership of the buffer backing the DrawTarget
    pub fn into_vec(self) -> Vec<u32> {
        self.buf
    }


    /// Saves the current pixel to a png file at `path`
    pub fn write_png<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), png::EncodingError> {
        let file = File::create(path)?;

        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        let mut output = Vec::with_capacity(self.buf.len() * 4);

        for pixel in &self.buf {
            let a = (pixel >> 24) & 0xffu32;
            let mut r = (pixel >> 16) & 0xffu32;
            let mut g = (pixel >> 8) & 0xffu32;
            let mut b = (pixel >> 0) & 0xffu32;

            if a > 0u32 {
                r = r * 255u32 / a;
                g = g * 255u32 / a;
                b = b * 255u32 / a;
            }

            output.push(r as u8);
            output.push(g as u8);
            output.push(b as u8);
            output.push(a as u8);
        }

        writer.write_image_data(&output)
    }
}
