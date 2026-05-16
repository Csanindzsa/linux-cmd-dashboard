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
    pub source: ThemeSource,
    pub alacritty_config: Option<String>,
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub accent: String,
    pub transparent_background: bool,
    pub background_opacity: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    Alacritty,
    System,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveTheme {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub accent: String,
    pub transparent_background: bool,
    pub background_opacity: f32,
    pub prefer_dark: Option<bool>,
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
            source: ThemeSource::Alacritty,
            alacritty_config: None,
            foreground: "#d8dee9".to_string(),
            background: "#111318".to_string(),
            cursor: "#f2f4f8".to_string(),
            accent: "#4cc9f0".to_string(),
            transparent_background: true,
            background_opacity: 0.8,
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

    pub fn effective_theme(&self, system_is_dark: bool) -> EffectiveTheme {
        match self.theme.source {
            ThemeSource::Alacritty => {
                let imported = self
                    .theme
                    .alacritty_path()
                    .and_then(|path| fs::read_to_string(path).ok())
                    .and_then(|raw| parse_alacritty_theme(&raw));

                imported
                    .map(|mut theme| {
                        if !self.theme.transparent_background {
                            theme.transparent_background = false;
                        }
                        theme
                    })
                    .unwrap_or_else(|| self.theme.custom_effective(Some(system_is_dark)))
            }
            ThemeSource::System => {
                let mut theme = self.theme.custom_effective(Some(system_is_dark));
                if system_is_dark {
                    theme.foreground = "#f2f4f8".to_string();
                    theme.background = "#1e1e1e".to_string();
                    theme.cursor = "#f2f4f8".to_string();
                    theme.accent = "#62a0ea".to_string();
                } else {
                    theme.foreground = "#1f2937".to_string();
                    theme.background = "#fafafa".to_string();
                    theme.cursor = "#111827".to_string();
                    theme.accent = "#1c71d8".to_string();
                }
                theme
            }
            ThemeSource::Custom => self.theme.custom_effective(None),
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("linux-cmd-dashboard")
        .join("config.toml")
}

impl Theme {
    fn alacritty_path(&self) -> Option<PathBuf> {
        self.alacritty_config
            .as_deref()
            .map(expand_home)
            .or_else(default_alacritty_path)
    }

    fn custom_effective(&self, prefer_dark: Option<bool>) -> EffectiveTheme {
        EffectiveTheme {
            foreground: self.foreground.clone(),
            background: self.background.clone(),
            cursor: self.cursor.clone(),
            accent: self.accent.clone(),
            transparent_background: self.transparent_background,
            background_opacity: self.background_opacity.clamp(0.1, 1.0),
            prefer_dark,
        }
    }
}

fn default_alacritty_path() -> Option<PathBuf> {
    dirs::config_dir()
        .map(|dir| dir.join("alacritty").join("alacritty.toml"))
        .filter(|path| path.exists())
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest)
    } else {
        PathBuf::from(path)
    }
}

pub fn parse_alacritty_theme(raw: &str) -> Option<EffectiveTheme> {
    let value: toml::Value = raw.parse().ok()?;
    let colors = value.get("colors")?;
    let primary = colors.get("primary")?;
    let foreground = color_value(primary.get("foreground"))?;
    let background = color_value(primary.get("background"))?;
    let cursor = colors
        .get("cursor")
        .and_then(|cursor| color_value(cursor.get("cursor")))
        .unwrap_or_else(|| foreground.clone());
    let accent = colors
        .get("bright")
        .and_then(|bright| color_value(bright.get("cyan")))
        .or_else(|| {
            colors
                .get("normal")
                .and_then(|normal| color_value(normal.get("cyan")))
        })
        .unwrap_or_else(|| "#88c0d0".to_string());
    let background_opacity = value
        .get("window")
        .and_then(|window| window.get("opacity"))
        .and_then(toml_number_to_f32)
        .unwrap_or(1.0)
        .clamp(0.1, 1.0);
    let prefer_dark = value
        .get("window")
        .and_then(|window| window.get("decorations_theme_variant"))
        .and_then(toml::Value::as_str)
        .and_then(|variant| match variant.to_ascii_lowercase().as_str() {
            "dark" => Some(true),
            "light" => Some(false),
            _ => None,
        });

    Some(EffectiveTheme {
        foreground,
        background,
        cursor,
        accent,
        transparent_background: background_opacity < 1.0,
        background_opacity,
        prefer_dark,
    })
}

fn color_value(value: Option<&toml::Value>) -> Option<String> {
    let raw = value?.as_str()?.trim();
    if let Some(hex) = raw.strip_prefix("0x") {
        Some(format!("#{hex}"))
    } else if raw.starts_with('#') {
        Some(raw.to_string())
    } else {
        None
    }
}

fn toml_number_to_f32(value: &toml::Value) -> Option<f32> {
    value
        .as_float()
        .map(|value| value as f32)
        .or_else(|| value.as_integer().map(|value| value as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_fish_and_expected_shortcuts() {
        let config = TerminalConfig::default();

        assert_eq!(config.shell, "fish");
        assert_eq!(config.theme.source, ThemeSource::Alacritty);
        assert!(config.theme.transparent_background);
        assert_eq!(config.theme.background_opacity, 0.8);
        assert_eq!(config.keybindings.new_pane, "<Ctrl><Shift>Return");
        assert_eq!(config.keybindings.restart_pane, "<Ctrl><Shift>r");
        assert!(config.inherit_focused_cwd);
    }

    #[test]
    fn missing_config_fields_fall_back_to_defaults() {
        let parsed: TerminalConfig = toml::from_str("shell = 'bash'\n").unwrap();

        assert_eq!(parsed.shell, "bash");
        assert_eq!(parsed.font, TerminalConfig::default().font);
        assert_eq!(parsed.theme.source, ThemeSource::Alacritty);
        assert_eq!(parsed.keybindings.restart_pane, "<Ctrl><Shift>r");
    }

    #[test]
    fn parses_alacritty_toml_theme() {
        let raw = r##"
            [window]
            opacity = 0.8
            decorations_theme_variant = "Dark"

            [colors.primary]
            background = "0x2E3440"
            foreground = "#D8DEE9"

            [colors.normal]
            cyan = "0x88C0D0"

            [colors.bright]
            cyan = "0x8FBCBB"
        "##;

        let theme = parse_alacritty_theme(raw).unwrap();

        assert_eq!(theme.background, "#2E3440");
        assert_eq!(theme.foreground, "#D8DEE9");
        assert_eq!(theme.cursor, "#D8DEE9");
        assert_eq!(theme.accent, "#8FBCBB");
        assert_eq!(theme.background_opacity, 0.8);
        assert!(theme.transparent_background);
        assert_eq!(theme.prefer_dark, Some(true));
    }

    #[test]
    fn config_round_trips_through_toml() {
        let config = TerminalConfig::default();

        let raw = toml::to_string(&config).unwrap();
        let parsed: TerminalConfig = toml::from_str(&raw).unwrap();

        assert_eq!(parsed, config);
    }
}
