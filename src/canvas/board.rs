use iced::{
    Point, Rectangle, Renderer, Theme,
    advanced::graphics::geometry::Frame,
    keyboard, mouse,
    widget::canvas::{self, Geometry, Path, Stroke},
    window,
};

use crate::canvas::{BoardCapsule, Message};

pub struct Board<'a> {
    pub app: &'a BoardCapsule,
}

/// Interaction state owned by the canvas widget.
#[derive(Default)]
pub enum Interaction {
    #[default]
    Idle,
    Panning {
        last: Point,
    },
    Dragging {
        index: usize,
        last: Point,
    },
}

impl Board<'_> {
    /// Topmost item under a screen-space point.
    fn hit_test(&self, screen: Point) -> Option<usize> {
        let world = self.app.camera.screen_to_world(screen);
        self.app
            .items
            .iter()
            .rposition(|item| item.bounds.contains(world))
    }
}

impl canvas::Program<Message> for Board<'_> {
    type State = Interaction;

    fn update(
        &self,
        state: &mut Interaction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        use canvas::Event;

        if let Event::Window(window::Event::FileDropped(path)) = event {
            return Some(canvas::Action::publish(Message::FileDropped(path.clone())));
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event
            && *key == keyboard::Key::Character("r".into())
        {
            return Some(canvas::Action::publish(Message::ResetView));
        }

        // Mouse release must end a drag even if it happens outside the canvas.
        if let Event::Mouse(mouse::Event::ButtonReleased(_)) = event
            && !matches!(state, Interaction::Idle)
        {
            *state = Interaction::Idle;
            return Some(canvas::Action::request_redraw().and_capture());
        }

        // Cursor relative to the canvas; while dragging, follow it outside too.
        let position = match state {
            Interaction::Idle => cursor.position_in(bounds)?,
            _ => cursor.position_from(bounds.position())?,
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(index) = self.hit_test(position) {
                    // The item will be moved to the end of the list.
                    let top = self.app.items.len() - 1;
                    *state = Interaction::Dragging {
                        index: top,
                        last: position,
                    };
                    Some(canvas::Action::publish(Message::BringToFront(index)).and_capture())
                } else {
                    *state = Interaction::Panning { last: position };
                    Some(canvas::Action::request_redraw().and_capture())
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                *state = Interaction::Panning { last: position };
                Some(canvas::Action::request_redraw().and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => match state {
                Interaction::Panning { last } => {
                    let delta = position - *last;
                    *last = position;
                    Some(canvas::Action::publish(Message::Pan(delta)).and_capture())
                }
                Interaction::Dragging { index, last } => {
                    let delta = position - *last;
                    *last = position;
                    Some(
                        canvas::Action::publish(Message::MoveItem {
                            index: *index,
                            delta,
                        })
                        .and_capture(),
                    )
                }
                Interaction::Idle => Some(canvas::Action::publish(Message::CursorMoved(position))),
            },
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let lines = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 60.0,
                };
                let factor = 1.15_f32.powf(lines);
                Some(
                    canvas::Action::publish(Message::Zoom {
                        factor,
                        anchor: position,
                    })
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Interaction,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let camera = &self.app.camera;
        let palette = theme.extended_palette();
        let mut frame = Frame::new(renderer, bounds.size());

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), palette.background.base.color);
        // draw_grid(
        //     &mut frame,
        //     camera,
        //     bounds.size(),
        //     palette.background.strong.color,
        // );

        let visible = camera.get_visible_area(bounds.size());
        let hovered = match state {
            Interaction::Dragging { index, .. } => Some(*index),
            _ => cursor.position_in(bounds).and_then(|p| self.hit_test(p)),
        };

        for (i, item) in self.app.items.iter().enumerate() {
            // Skip anything off-screen; this is what keeps a huge board fast.
            if !visible.intersects(&item.bounds) {
                continue;
            }

            let top_left = camera.world_to_screen(item.bounds.position());
            let size = item.bounds.size() * camera.zoom;
            let screen = Rectangle::new(top_left, size);

            frame.draw_image(screen, canvas::Image::new(item.handle.clone()));

            if hovered == Some(i) {
                frame.stroke(
                    &Path::rectangle(top_left, size),
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(palette.primary.base.color),
                );
            }
        }

        // Zoom indicator
        frame.fill_text(canvas::Text {
            content: format!(
                "{:.0}%  ·  {} images",
                camera.zoom * 100.0,
                self.app.items.len()
            ),
            position: Point::new(12.0, bounds.height - 28.0),
            color: palette.background.base.text.scale_alpha(0.6),
            size: 14.0.into(),
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Interaction,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match state {
            Interaction::Panning { .. } | Interaction::Dragging { .. } => {
                mouse::Interaction::Grabbing
            }
            Interaction::Idle => match cursor.position_in(bounds) {
                Some(p) if self.hit_test(p).is_some() => mouse::Interaction::Grab,
                _ => mouse::Interaction::default(),
            },
        }
    }
}
