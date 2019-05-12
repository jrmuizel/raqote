mod blitter;
mod dash;
mod draw_target;
mod geom;
mod rasterizer;
mod stroke;
mod tests;

mod path_builder;
pub use path_builder::*;

pub use crate::draw_target::{BlendMode, DrawOptions, DrawTarget, SolidSource, Source, Winding};
pub use crate::stroke::*;

pub use sw_composite::{Gradient, GradientStop, Image};

pub type IntRect = euclid::Box2D<i32>;
pub type Point = euclid::Point2D<f32>;
pub type Transform = euclid::Transform2D<f32>;
pub type Vector = euclid::Vector2D<f32>;
