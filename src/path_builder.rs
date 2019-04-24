use crate::types::Point;
use crate::rasterizer::Rasterizer;
use crate::geom::*;


use lyon_geom::cubic_to_quadratic::cubic_to_quadratics;
use lyon_geom::CubicBezierSegment;
use euclid::Point2D;

pub struct PathBuilder<'a> {

    rasterizer: &'a mut Rasterizer,
}

impl<'a> PathBuilder<'a> {
    pub fn new(rasterizer: &'a mut Rasterizer) -> PathBuilder<'a> {
        PathBuilder {

            rasterizer, }
    }

}