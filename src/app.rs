use std::time::{Duration, Instant};

use iced::{Element, Subscription, Task, time, widget::text, window};

use crate::config::WindowConfig;

#[derive(Default, Debug)]
pub struct OxiRef {
    pub config: WindowConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    Window(window::Event),
    Tick(Instant),
}

impl OxiRef {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if let Err(e) = self.config.save_if_needed(&now) {
                    eprintln!("Failed to save window state: {e}");
                }
            }
            Message::Window(window::Event::Moved(p)) => {
                if p.x > -10_000.0 && p.y > -10_000.0 {
                    self.config.set_position(Some((p.x, p.y)));
                }
            }
            Message::Window(window::Event::Resized(s)) => {
                if s.width > 0.0 && s.height > 0.0 {
                    self.config.set_size((s.width, s.height));
                }
            }
            Message::Window(window::Event::CloseRequested) => {
                if let Err(e) = self.config.save() {
                    eprintln!("Failed to save window state: {e}");
                }
                return iced::exit();
            }
            Message::Window(_) => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        text("OxiRef").into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let window_event = window::events().map(|(_id, event)| Message::Window(event));

        Subscription::batch([
            window_event,
            time::every(Duration::from_millis(16)).map(Message::Tick),
        ])
    }
}
