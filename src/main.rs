mod app;
mod config;
mod constants;

use crate::app::OxiRef;
use crate::config::WindowConfig;
use iced::{Point, Size, Task, Theme, window};

pub fn main() -> iced::Result {
    let config = WindowConfig::load();
    let size = Size::new(config.size().0, config.size().1);

    let position = config
        .position()
        .map(|(x, y)| window::Position::Specific(Point::new(x, y)))
        .unwrap_or(window::Position::Centered);

    iced::application(
        move || {
            (
                OxiRef {
                    config: config.clone(),
                },
                Task::none(),
            )
        },
        OxiRef::update,
        OxiRef::view,
    )
    .subscription(OxiRef::subscription)
    .window_size(size)
    .position(position)
    .exit_on_close_request(false)
    .theme(Theme::Dracula)
    .run()
}
