mod rasterizer;
mod types;
mod geom;
mod blitter;
mod draw_target;
mod stroke;
mod dash;
mod tests;

mod path_builder;
pub use path_builder::PathBuilder;

pub use crate::draw_target::{DrawTarget, Source, SolidSource, Winding};
pub use crate::stroke::*;

pub use sw_composite::{GradientStop, Gradient, Image};
use std::collections::hash_map::DefaultHasher;

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;





