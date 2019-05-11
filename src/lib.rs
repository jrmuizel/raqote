mod blitter;
mod dash;
mod draw_target;
mod geom;
mod rasterizer;
mod stroke;
mod tests;

mod path_builder;
pub use path_builder::*;

pub use crate::draw_target::{DrawTarget, SolidSource, Source, Winding, BlendMode};
pub use crate::stroke::*;

use std::collections::hash_map::DefaultHasher;
pub use sw_composite::{Gradient, GradientStop, Image};

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

pub type IntRect = euclid::Box2D<i32>;
pub type Point = euclid::Point2D<f32>;
pub type Transform = euclid::Transform2D<f32>;
pub type Vector = euclid::Vector2D<f32>;
