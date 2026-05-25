use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};

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
    pub kitty_config: Option<String>,
    pub foreground: String,
    pub background: String,
    pub titlebar_background: String,
    pub cursor: String,
    pub accent: String,
    pub transparent_background: bool,
    pub background_opacity: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    Alacritty,
    Kitty,
    System,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveTheme {
    pub foreground: String,
    pub background: String,
    pub titlebar_background: String,
    pub cursor: String,
    pub accent: String,
    pub transparent_background: bool,
    pub background_opacity: f32,
    pub prefer_dark: Option<bool>,
    pub ansi_colors: Vec<String>,
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
            kitty_config: None,
            foreground: "#6D8096".to_string(),
            background: "#282a36".to_string(),
            titlebar_background: "#191a21".to_string(),
            cursor: "#f8f8f2".to_string(),
            accent: "#8be9fd".to_string(),
            transparent_background: true,
            background_opacity: 0.7,
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
                        apply_imported_theme_opacity(&mut theme, &self.theme);
                        theme
                    })
                    .unwrap_or_else(|| self.theme.custom_effective(Some(system_is_dark)))
            }
            ThemeSource::Kitty => {
                let imported = self
                    .theme
                    .kitty_theme_data()
                    .and_then(|raw| parse_kitty_theme(&raw));

                imported
                    .map(|mut theme| {
                        apply_imported_theme_opacity(&mut theme, &self.theme);
                        theme
                    })
                    .unwrap_or_else(|| self.theme.custom_effective(Some(system_is_dark)))
            }
            ThemeSource::System => {
                let mut theme = self.theme.custom_effective(Some(system_is_dark));
                if system_is_dark {
                    theme.foreground = "#6D8096".to_string();
                    theme.background = "#282a36".to_string();
                    theme.titlebar_background = "#191a21".to_string();
                    theme.cursor = "#f8f8f2".to_string();
                    theme.accent = "#8be9fd".to_string();
                } else {
                    theme.foreground = "#1f2937".to_string();
                    theme.background = "#fafafa".to_string();
                    theme.titlebar_background = "#e5e7eb".to_string();
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

    fn kitty_theme_data(&self) -> Option<String> {
        resolve_kitty_theme_source(self.kitty_config.as_deref())
    }

    fn custom_effective(&self, prefer_dark: Option<bool>) -> EffectiveTheme {
        EffectiveTheme {
            foreground: self.foreground.clone(),
            background: self.background.clone(),
            titlebar_background: self.titlebar_background.clone(),
            cursor: self.cursor.clone(),
            accent: self.accent.clone(),
            transparent_background: self.transparent_background,
            background_opacity: self.background_opacity.clamp(0.1, 1.0),
            prefer_dark,
            ansi_colors: Vec::new(),
        }
    }
}

fn apply_imported_theme_opacity(theme: &mut EffectiveTheme, source: &Theme) {
    let configured_opacity = source.background_opacity.clamp(0.1, 1.0);
    if source.transparent_background {
        theme.background_opacity = configured_opacity;
        theme.transparent_background = true;
    } else {
        theme.transparent_background = false;
        theme.background_opacity = 1.0;
    }
}

const ANSICOLOR_ORDER: [(&str, usize); 16] = [
    ("black", 0),
    ("red", 1),
    ("green", 2),
    ("yellow", 3),
    ("blue", 4),
    ("magenta", 5),
    ("cyan", 6),
    ("white", 7),
    ("black", 8),
    ("red", 9),
    ("green", 10),
    ("yellow", 11),
    ("blue", 12),
    ("magenta", 13),
    ("cyan", 14),
    ("white", 15),
];

const ANSI_DEFAULTS: [&str; 16] = [
    "#000000", "#D32E2E", "#28A745", "#F0AD4E", "#268BD2", "#8A3FFC", "#2AA198", "#BEBEBE",
    "#586e75", "#DC322F", "#859900", "#B58900", "#6C71C4", "#D33682", "#2AA198", "#FDF6E3",
];

fn parse_alacritty_ansi_colors(colors: &toml::Value) -> Vec<String> {
    let mut ansi_colors: Vec<String> = ANSI_DEFAULTS
        .iter()
        .map(|value| (*value).to_string())
        .collect();

    if let Some(normal) = colors.get("normal") {
        for &(name, index) in ANSICOLOR_ORDER[..8].iter() {
            if let Some(value) = color_value(normal.get(name)) {
                ansi_colors[index] = value;
            }
        }
    }

    if let Some(bright) = colors.get("bright") {
        for &(name, index) in ANSICOLOR_ORDER[8..].iter() {
            if let Some(value) = color_value(bright.get(name)) {
                ansi_colors[index] = value;
            }
        }
    }

    ansi_colors
}

fn parse_kitty_theme(raw: &str) -> Option<EffectiveTheme> {
    let mut foreground = None;
    let mut background = None;
    let mut cursor = None;
    let mut accent = None;
    let mut titlebar_background = None;
    let mut background_opacity = 1.0;
    let mut ansi_colors: Vec<String> = ANSI_DEFAULTS
        .iter()
        .map(|value| (*value).to_string())
        .collect();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = parse_key_value_pair(line) else {
            continue;
        };
        let value = value.trim_matches('"');

        match key {
            "foreground" => foreground = parse_kitty_color(value),
            "background" => {
                if let Some(color) = parse_kitty_color(value) {
                    background = Some(color.clone());
                    titlebar_background = Some(color);
                }
            }
            "cursor" => cursor = parse_kitty_color(value),
            "cursor_text_color" => accent = parse_kitty_color(value),
            "background_opacity" => background_opacity = parse_numeric(value).unwrap_or(1.0),
            _ => {
                if let Some((prefix, index)) = key.split_once("color") {
                    if prefix.is_empty() {
                        if let Ok(index) = index.parse::<usize>() {
                            if index < ansi_colors.len() {
                                if let Some(color) = parse_kitty_color(value) {
                                    ansi_colors[index] = color;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let foreground = foreground.unwrap_or_else(|| "#ffffff".to_string());
    let background = background?;
    let cursor = cursor.unwrap_or_else(|| foreground.clone());
    let accent = accent.unwrap_or_else(|| ansi_colors[6].clone());
    let prefer_dark = Some(is_dark_background(&background));
    let background_opacity = background_opacity.clamp(0.1, 1.0);

    Some(EffectiveTheme {
        foreground,
        background: background.clone(),
        titlebar_background: titlebar_background.unwrap_or(background),
        cursor,
        accent,
        transparent_background: background_opacity < 1.0,
        background_opacity,
        prefer_dark,
        ansi_colors,
    })
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
        background: background.clone(),
        titlebar_background: background,
        cursor,
        accent,
        transparent_background: background_opacity < 1.0,
        background_opacity,
        prefer_dark,
        ansi_colors: parse_alacritty_ansi_colors(colors),
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

fn parse_key_value_pair(line: &str) -> Option<(&str, &str)> {
    if let Some((key, value)) = line.split_once('=') {
        Some((key.trim(), value.trim()))
    } else {
        let mut parts = line.split_whitespace();
        let key = parts.next()?;
        let value = parts.next()?;
        Some((key, value))
    }
}

fn parse_numeric(raw: &str) -> Option<f32> {
    raw.parse::<f32>().ok()
}

fn parse_kitty_color(raw: &str) -> Option<String> {
    let value = raw.trim();
    if let Some(hex) = value.strip_prefix("0x") {
        return Some(format!("#{hex}"));
    }
    if let Some(hex) = value.strip_prefix('#') {
        return normalize_hex_alpha(hex);
    }
    if let Some(rgb) = value.strip_prefix("rgb:") {
        let mut parts = rgb.split('/');
        let red = parts.next()?;
        let green = parts.next()?;
        let blue = parts.next()?;

        let red = normalize_kitty_rgb_part(red)?;
        let green = normalize_kitty_rgb_part(green)?;
        let blue = normalize_kitty_rgb_part(blue)?;

        return Some(format!("#{red}{green}{blue}"));
    }

    None
}

fn normalize_kitty_rgb_part(part: &str) -> Option<String> {
    match part.len() {
        1 => Some(format!("{value}{value}", value = part)),
        2 => Some(part.to_string()),
        4 => Some(part[..2].to_string()),
        _ => None,
    }
}

fn normalize_hex_alpha(raw: &str) -> Option<String> {
    match raw.len() {
        3 => Some(format!(
            "#{0}{0}{1}{1}{2}{2}",
            &raw[0..1],
            &raw[1..2],
            &raw[2..3]
        )),
        4 => Some(format!(
            "#{0}{0}{1}{1}{2}{2}",
            &raw[0..1],
            &raw[1..2],
            &raw[2..3]
        )),
        6 => Some(format!("#{raw}")),
        8 => Some(format!("#{}", &raw[..6])),
        _ => None,
    }
}

fn is_dark_background(background: &str) -> bool {
    let raw = background.trim_start_matches('#');
    if raw.len() < 6 {
        return true;
    }
    let r = u8::from_str_radix(&raw[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&raw[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&raw[4..6], 16).unwrap_or(0);
    let luminance = 0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b);
    luminance < 128.0
}

fn resolve_kitty_theme_source(spec: Option<&str>) -> Option<String> {
    let spec = spec.unwrap_or("");
    let trimmed = spec.trim();

    if trimmed.is_empty() {
        return default_kitty_path().and_then(|path| fs::read_to_string(path).ok());
    }

    let explicit_path = expand_home(trimmed);
    if explicit_path.exists() {
        return fs::read_to_string(explicit_path).ok();
    }

    if let Some(path) = explicit_kitty_theme_file(trimmed) {
        if path.exists() {
            return fs::read_to_string(path).ok();
        }
    }

    run_kitty_dump_theme(trimmed)
}

fn default_kitty_path() -> Option<PathBuf> {
    dirs::config_dir()
        .map(|dir| dir.join("kitty").join("kitty.conf"))
        .filter(|path| path.exists())
}

fn explicit_kitty_theme_file(name: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|dir| {
        let filename = if name.ends_with(".conf") {
            name.to_string()
        } else {
            format!("{name}.conf")
        };
        dir.join("kitty").join("themes").join(filename)
    })
}

fn run_kitty_dump_theme(name: &str) -> Option<String> {
    let output = Command::new("kitty")
        .arg("+kitten")
        .arg("themes")
        .arg("--dump-theme")
        .arg(name)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
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
        assert_eq!(config.theme.background_opacity, 0.7);
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
            black = "0x181818"
            red = "0xA54242"
            green = "0x8C9440"
            yellow = "0xDE935F"
            blue = "0x5F819D"
            magenta = "0x85678F"
            white = "0x707880"
            cyan = "0x88C0D0"

            [colors.bright]
            black = "0x000000"
            red = "0xF0C674"
            green = "0xB5BD68"
            yellow = "0xF0C674"
            blue = "0x81A2BE"
            magenta = "0xB294BB"
            cyan = "0x8FBCBB"
            white = "0xFFFFFF"
        "##;

        let theme = parse_alacritty_theme(raw).unwrap();

        assert_eq!(theme.background, "#2E3440");
        assert_eq!(theme.foreground, "#D8DEE9");
        assert_eq!(theme.cursor, "#D8DEE9");
        assert_eq!(theme.accent, "#8FBCBB");
        assert_eq!(theme.ansi_colors[0], "#181818");
        assert_eq!(theme.ansi_colors[1], "#A54242");
        assert_eq!(theme.ansi_colors[8], "#000000");
        assert_eq!(theme.ansi_colors[14], "#8FBCBB");
        assert_eq!(theme.background_opacity, 0.8);
        assert!(theme.transparent_background);
        assert_eq!(theme.prefer_dark, Some(true));
    }

    #[test]
    fn parses_kitty_theme() {
        let raw = r#"
            foreground #ebdbb2
            background #282828
            cursor #ebdbb2
            background_opacity 0.75
            color0 #282828
            color1 #cc241d
            color7 #928374
            color15 #fbf1c7
        "#;

        let theme = parse_kitty_theme(raw).unwrap();

        assert_eq!(theme.foreground, "#ebdbb2");
        assert_eq!(theme.background, "#282828");
        assert_eq!(theme.cursor, "#ebdbb2");
        assert_eq!(theme.ansi_colors[0], "#282828");
        assert_eq!(theme.ansi_colors[1], "#cc241d");
        assert_eq!(theme.ansi_colors[7], "#928374");
        assert_eq!(theme.ansi_colors[15], "#fbf1c7");
        assert_eq!(theme.background_opacity, 0.75);
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
