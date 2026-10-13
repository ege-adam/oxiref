use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::{fs, path};

use serde::{Deserialize, Serialize};

use directories::{ProjectDirs, UserDirs};
use iced::widget::{button, text};
use iced::{Element, Point, Size, Subscription, Task, Theme, window};

mod constants;

pub fn main() -> iced::Result {
    let config = WindowConfig::load();
    let size = Size::new(config.size.0, config.size.1);

    let position = config
        .position
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
    .theme(Theme::Dracula)
    .run()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowConfig {
    size: (f32, f32),
    position: Option<(f32, f32)>,

    #[serde(skip)]
    dirty_since: Option<Instant>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            size: (800.0, 600.0),
            position: None,
            dirty_since: Some(Instant::now()),
        }
    }
}

impl WindowConfig {
    fn path() -> Option<PathBuf> {
        ProjectDirs::from("", "", constants::SAVE_DIR_NAME)
            .map(|d| d.config_dir().join(constants::SAVE_FILE_NAME))
    }

    fn load() -> Self {
        let path = match Self::path() {
            Some(path) => path,
            None => {
                return Self::default();
            }
        };

        let toml_content = match fs::read_to_string(&path) {
            Ok(content) => Some(content),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => None,
                _ => {
                    println!(
                        "Error when reading from file at {}, with error {}",
                        &path.display(),
                        e
                    );
                    None
                }
            },
        };

        match toml_content {
            None => Self::default(),
            Some(content) => toml::from_str(&content).unwrap_or(Self::default()),
        }
    }

    fn save(&self) -> io::Result<()> {
        let path = Self::path().ok_or_else(|| io::Error::other("no config dir"))?;

        if let Some(dir) = &path.parent() {
            fs::create_dir_all(dir)?;
        }

        fs::write(
            path,
            toml::to_string_pretty(self).map_err(io::Error::other)?,
        )
    }
}

#[derive(Default, Debug)]
struct OxiRef {
    config: WindowConfig,
}

#[derive(Debug, Clone)]
enum Message {
    Window(window::Event),
    Tick(Instant),
}

impl OxiRef {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if self
                    .config
                    .dirty_since
                    .is_some_and(|t| now.duration_since(t) >= constants::SAVE_DEBOUNCE)
                {
                    match self.config.save() {
                        Err(e) => {
                            println!("Error when saving config {e}");
                        }
                        _ => (),
                    }
                }
            }
            Message::Window(window::Event::Moved(p)) => {
                if p.x > -10_000.0 && p.y > -10_000.0 {
                    self.config.position = Some((p.x, p.y));
                    self.config.dirty_since = Some(Instant::now());
                }
            }
            Message::Window(window::Event::Resized(s)) => {
                if s.width > 0.0 && s.height > 0.0 {
                    self.config.size = (s.width, s.height);
                    self.config.dirty_since = Some(Instant::now());
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

    fn view(&self) -> Element<'_, Message> {
        text("OxiRef").into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::events().map(|(_id, event)| Message::Window(event))
    }
}
