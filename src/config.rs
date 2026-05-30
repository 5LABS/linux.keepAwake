use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SavedMode {
    #[default]
    Off,
    Indefinite,
    Timed { secs: u64 },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keep_screen_on: bool,
    #[serde(default)]
    pub mode: SavedMode,
}

fn config_file() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("keep-awake").join("config.toml"))
}

pub fn load() -> Config {
    let Some(path) = config_file() else {
        return Config::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save(cfg: &Config) {
    let Some(path) = config_file() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(s) = toml::to_string_pretty(cfg) {
        if let Err(e) = std::fs::write(&path, s) {
            eprintln!("keep-awake: could not save config: {e}");
        }
    }
}

fn autostart_file() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("autostart").join("keep-awake.desktop"))
}

pub fn autostart_enabled() -> bool {
    autostart_file().map(|p| p.exists()).unwrap_or(false)
}

/// Create or remove the XDG autostart entry. The `Exec` line points at the
/// currently running binary so it works regardless of install location.
pub fn set_autostart(enabled: bool) {
    let Some(path) = autostart_file() else { return };
    if enabled {
        let exec = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(str::to_owned))
            .unwrap_or_else(|| "keep-awake".to_owned());
        let entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Keep Awake\n\
             Comment=Keep the system and screen awake\n\
             Exec={exec}\n\
             Icon=keep-awake\n\
             Terminal=false\n\
             Categories=Utility;\n\
             X-GNOME-Autostart-enabled=true\n\
             X-GNOME-Autostart-Delay=3\n"
        );
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&path, entry) {
            eprintln!("keep-awake: could not write autostart entry: {e}");
        }
    } else if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            eprintln!("keep-awake: could not remove autostart entry: {e}");
        }
    }
}
