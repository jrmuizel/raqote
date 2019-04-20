use crate::types::Point;
use crate::rasterizer::Rasterizer;

pub struct PathBuilder<'a, 'b> {
    current_point: Point,
    first_point: Point,
    rasterizer: &'b mut Rasterizer<'a >,
}

impl<'a, 'b> PathBuilder<'a, 'b> {
    pub fn new(rasterizer: &'b mut Rasterizer<'a>) -> PathBuilder<'a, 'b> {
        PathBuilder {
            current_point: Point { x: 0., y: 0.},
            first_point: Point { x: 0., y: 0. },
            rasterizer, }
    }
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.current_point = Point { x, y };
        self.first_point = Point { x, y };
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        let p = Point {x, y};
        self.rasterizer.add_edge(self.current_point, p, false, Point {x: 0., y: 0.});
        self.current_point = p;
    }

    pub fn close(&mut self) {
        self.rasterizer.add_edge(self.current_point, self.first_point, false, Point {x: 0., y: 0.});
    }
}