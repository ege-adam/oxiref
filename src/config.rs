use std::{fs, io, path::PathBuf, time::Instant};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::constants;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
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

    pub fn save_if_needed(&mut self, now: &Instant) -> Result<(), Box<dyn std::error::Error>> {
        if self
            .dirty_since
            .is_some_and(|t| now.duration_since(t) > constants::SAVE_DEBOUNCE)
        {
            self.save()?
        }

        return Ok(());
    }

    pub fn load() -> Self {
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

    pub fn save(&mut self) -> io::Result<()> {
        let path = Self::path().ok_or_else(|| io::Error::other("no config dir"))?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let tmp = path.with_extension("toml.tmp");
        fs::write(
            &tmp,
            toml::to_string_pretty(self).map_err(io::Error::other)?,
        )?;
        fs::rename(&tmp, &path)?;

        self.dirty_since = None;
        Ok(())
    }

    pub fn set_size(&mut self, size: (f32, f32)) {
        self.size = size;
        self.set_dirty();
    }

    pub fn size(&self) -> (f32, f32) {
        self.size
    }

    pub fn position(&self) -> Option<(f32, f32)> {
        self.position
    }

    pub fn set_position(&mut self, position: Option<(f32, f32)>) {
        self.position = position;
        self.set_dirty();
    }

    pub fn set_dirty(&mut self) {
        self.dirty_since = Some(Instant::now());
    }
}
