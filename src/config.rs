use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TerminalConfig {
    pub shell: String,
    pub inherit_focused_cwd: bool,
    pub font: String,
    pub theme: Theme,
    pub scrollback_lines: i64,
    pub keybindings: Keybindings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Theme {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Keybindings {
    pub new_pane: String,
    pub close_pane: String,
    pub restart_pane: String,
    pub focus_left: String,
    pub focus_down: String,
    pub focus_up: String,
    pub focus_right: String,
    pub move_left: String,
    pub move_down: String,
    pub move_up: String,
    pub move_right: String,
    pub fullscreen: String,
    pub overview: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: "fish".to_string(),
            inherit_focused_cwd: true,
            font: "Monospace 11".to_string(),
            theme: Theme::default(),
            scrollback_lines: 20_000,
            keybindings: Keybindings::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            foreground: "#d8dee9".to_string(),
            background: "#111318".to_string(),
            cursor: "#f2f4f8".to_string(),
            accent: "#4cc9f0".to_string(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            new_pane: "<Ctrl><Shift>Return".to_string(),
            close_pane: "<Ctrl><Shift>w".to_string(),
            restart_pane: "<Ctrl><Shift>r".to_string(),
            focus_left: "<Ctrl><Shift>h".to_string(),
            focus_down: "<Ctrl><Shift>j".to_string(),
            focus_up: "<Ctrl><Shift>k".to_string(),
            focus_right: "<Ctrl><Shift>l".to_string(),
            move_left: "<Ctrl><Shift><Alt>h".to_string(),
            move_down: "<Ctrl><Shift><Alt>j".to_string(),
            move_up: "<Ctrl><Shift><Alt>k".to_string(),
            move_right: "<Ctrl><Shift><Alt>l".to_string(),
            fullscreen: "<Ctrl><Shift>f".to_string(),
            overview: "<Ctrl><Shift>o".to_string(),
        }
    }
}

impl TerminalConfig {
    pub fn load() -> Self {
        Self::try_load().unwrap_or_default()
    }

    pub fn load_or_create() -> Self {
        let config = Self::load();
        if !config_path().exists() {
            let _ = config.save();
        }
        config
    }

    pub fn try_load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse config {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }

        let raw = toml::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(&path, raw).with_context(|| format!("failed to write config {}", path.display()))
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("linux-cmd-dashboard")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_fish_and_expected_shortcuts() {
        let config = TerminalConfig::default();

        assert_eq!(config.shell, "fish");
        assert_eq!(config.keybindings.new_pane, "<Ctrl><Shift>Return");
        assert_eq!(config.keybindings.restart_pane, "<Ctrl><Shift>r");
        assert!(config.inherit_focused_cwd);
    }

    #[test]
    fn missing_config_fields_fall_back_to_defaults() {
        let parsed: TerminalConfig = toml::from_str("shell = 'bash'\n").unwrap();

        assert_eq!(parsed.shell, "bash");
        assert_eq!(parsed.font, TerminalConfig::default().font);
        assert_eq!(parsed.keybindings.restart_pane, "<Ctrl><Shift>r");
    }

    #[test]
    fn config_round_trips_through_toml() {
        let config = TerminalConfig::default();

        let raw = toml::to_string(&config).unwrap();
        let parsed: TerminalConfig = toml::from_str(&raw).unwrap();

        assert_eq!(parsed, config);
    }
}
