use std::path::PathBuf;

use iced::{Element, Length::Fill, Point, Rectangle, Size, Vector, widget::Canvas};

use crate::canvas::{camera::Camera, item::Item};

mod board;
mod camera;
mod item;

#[derive(Debug, Clone)]
enum Message {
    Pan(Vector),
    Zoom { factor: f32, anchor: Point },
    MoveItem { index: usize, delta: Vector },
    BringToFront(usize),
    CursorMoved(Point),
    FileDropped(PathBuf),
    ResetView,
}

struct BoardCapsule {
    camera: Camera,
    items: Vec<Item>,
    cursor_world: Point,
}

impl BoardCapsule {
    fn update(&mut self, message: Message) {
        match message {
            Message::Pan(delta) => {
                self.camera.offset -= delta * (1.0 / self.camera.zoom);
            }
            Message::Zoom { factor, anchor } => self.camera.zoom_at(factor, anchor),
            Message::MoveItem { index, delta } => {
                if let Some(item) = self.items.get_mut(index) {
                    let d = delta * (1.0 / self.camera.zoom);
                    item.bounds.x += d.x;
                    item.bounds.y += d.y;
                }
            }
            Message::BringToFront(index) => {
                if index < self.items.len() {
                    let item = self.items.remove(index);
                    self.items.push(item);
                }
            }
            Message::CursorMoved(p) => self.cursor_world = self.camera.screen_to_world(p),
            Message::FileDropped(path) => match Item::load_image(&path) {
                Ok((handle, w, h)) => {
                    let size = Size::new(w as f32, h as f32);
                    let center = self.cursor_world;
                    self.items.push(Item {
                        handle,
                        bounds: Rectangle::new(
                            Point::new(center.x - size.width / 2.0, center.y - size.height / 2.0),
                            size,
                        ),
                        z_index: 0,
                    });
                }
                Err(err) => eprintln!("Could not load {}: {err}", path.display()),
            },
            Message::ResetView => self.camera = Camera::default(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let program = board::Board { app: self };
        Canvas::new(program).width(Fill).height(Fill).into()
    }
}
