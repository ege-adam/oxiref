use iced::{Point, Rectangle, Size, Vector};

pub struct Camera {
    pub offset: Vector,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offset: Vector::new(-100.0, -100.0),
            zoom: 1.0,
        }
    }
}

impl Camera {
    const MIN_ZOOM: f32 = 0.05;
    const MAX_ZOOM: f32 = 20.0;

    pub fn screen_to_world(&self, p: Point) -> Point {
        Point::new(
            p.x / self.zoom + self.offset.x,
            p.y / self.zoom + self.offset.y,
        )
    }

    pub fn world_to_screen(&self, p: Point) -> Point {
        Point::new(
            (p.x - self.offset.x) * self.zoom,
            (p.y - self.offset.y) * self.zoom,
        )
    }

    pub fn zoom_at(&mut self, factor: f32, anchor: Point) {
        let before = self.screen_to_world(anchor);
        self.zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let after = self.screen_to_world(anchor);
        self.offset += before - after;
    }

    pub fn get_visible_area(&self, size: Size) -> Rectangle {
        Rectangle::new(
            Point::new(self.offset.x, self.offset.y),
            Size::new(size.width / self.zoom, size.height / self.zoom),
        )
    }
}
