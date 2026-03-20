use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    pub fn all() -> [Direction; 8] {
        [
            Direction::N,
            Direction::NE,
            Direction::E,
            Direction::SE,
            Direction::S,
            Direction::SW,
            Direction::W,
            Direction::NW,
        ]
    }

    /// Map an angle in degrees (0 = East, counter-clockwise) to a direction.
    pub fn from_angle(degrees: f64) -> Direction {
        // Normalize to [0, 360)
        let d = ((degrees % 360.0) + 360.0) % 360.0;
        match d as u32 {
            338..=360 | 0..=22 => Direction::E,
            23..=67 => Direction::NE,
            68..=112 => Direction::N,
            113..=157 => Direction::NW,
            158..=202 => Direction::W,
            203..=247 => Direction::SW,
            248..=292 => Direction::S,
            _ => Direction::SE,
        }
    }
}

/// A keyboard shortcut described as a list of key names.
/// e.g. ["ctrl", "c"] or ["super", "tab"]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Minimum pixel distance from press origin before a gesture is recognised.
    pub threshold_px: f64,
    /// Per-direction shortcut. None means the direction is unbound.
    pub directions: HashMap<Direction, Option<Shortcut>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut directions = HashMap::new();
        for d in Direction::all() {
            directions.insert(d, None);
        }
        Config {
            threshold_px: 15.0,
            directions,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gestro")
        .join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str(&data) {
            return cfg;
        }
    }
    Config::default()
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}
