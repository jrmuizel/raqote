mod rasterizer;
mod types;
use typed_arena::Arena;
use types::Point;
fn main() {

    let mut arena = Arena::new();
    let mut r = rasterizer::Rasterizer::new(&mut arena, 400, 400);
    r.add_edge(Point{x: 50., y: 50.}, Point{x: 100., y: 70.}, false, Point{x: 0., y: 0.});
    r.add_edge(Point{x: 100., y: 70.}, Point{x: 110., y: 150.}, false, Point{x: 0., y: 0.});
    r.add_edge(Point{x: 110., y: 150.}, Point{x: 40., y: 180.}, false, Point{x: 0., y: 0.});
    r.add_edge(Point{x: 40., y: 180.}, Point{x: 50., y: 50.}, false, Point{x: 0., y: 0.});
    r.rasterize();
}
