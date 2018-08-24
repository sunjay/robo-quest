use nalgebra::Vector2;
use sdl2::rect::Point;

pub type Vec2D = Vector2<f64>;

pub trait ToVec2D {
    fn to_vec2d(self) -> Vec2D;
}

impl ToVec2D for Point {
    fn to_vec2d(self) -> Vec2D {
        Vec2D::new(self.x() as f64, self.y() as f64)
    }
}

pub trait ToPoint {
    fn to_point(self) -> Point;
}

impl ToPoint for Vec2D {
    fn to_point(self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}
