use glam::{vec2, Vec2};
use iced::{Point, Vector};

pub trait GeometryConversion {
    fn vector(self) -> Vector;
    fn vec(self) -> Vec2;
    fn point(self) -> Point;
}

impl GeometryConversion for Vec2 {
    fn vector(self) -> Vector {
        Vector::new(self.x, self.y)
    }

    fn vec(self) -> Vec2 {
        self
    }

    fn point(self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl GeometryConversion for Vector {
    fn vector(self) -> Vector {
        self
    }

    fn vec(self) -> Vec2 {
        vec2(self.x, self.y)
    }

    fn point(self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl GeometryConversion for Point {
    fn vector(self) -> Vector {
        Vector::new(self.x, self.y)
    }

    fn vec(self) -> Vec2 {
        vec2(self.x, self.y)
    }

    fn point(self) -> Point {
        self
    }
}
