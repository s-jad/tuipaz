use std::{fs, num::ParseIntError, fmt::{Display, self}, error::Error};

use ratatui::style::{Color, self};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub(crate) struct UserColors {
    black: Option<String>,
    red: Option<String>,
    green: Option<String>,
    yellow: Option<String>,
    blue: Option<String>,
    magenta: Option<String>,
    cyan: Option<String>,
    white: Option<String>,
    bright_black: Option<String>,
    bright_red: Option<String>,
    bright_green: Option<String>,
    bright_yellow: Option<String>,
    bright_blue: Option<String>,
    bright_magenta: Option<String>,
    bright_cyan: Option<String>,
    bright_white: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Colors {
    black: Color,
    red: Color,
    green: Color,
    yellow: Color,
    blue: Color,
    magenta: Color,
    cyan: Color,
    white: Color,
    bright_black: Color,
    bright_red: Color,
    bright_green: Color,
    bright_yellow: Color,
    bright_blue: Color,
    bright_magenta: Color,
    bright_cyan: Color,
    bright_white: Color,
}

#[derive(Debug)]
enum HexColorError {
    InvalidFormat,
    ParseError(ParseIntError),
}

impl From<ParseIntError> for HexColorError {
    fn from(err: ParseIntError) -> HexColorError {
        HexColorError::ParseError(err)
    }
}

#[derive(Debug)]
pub(crate) struct ColorConversionError {
    message: String,
}

impl Display for ColorConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn hex_to_color(hex: &str) -> Result<style::Color, HexColorError> {
    if hex.len()!= 7 ||!hex.starts_with('#') {
        return Err(HexColorError::InvalidFormat);
    }
    let r = u8::from_str_radix(&hex[1..3], 16)?;
    let g = u8::from_str_radix(&hex[3..5], 16)?;
    let b = u8::from_str_radix(&hex[5..7], 16)?;
    Ok(style::Color::Rgb(r, g, b))
}

fn try_convert_color(user_color: Option<String>, default: Color) -> Result<Color, ColorConversionError> {
    if let Some(c) = user_color {
        match hex_to_color(&c) {
            Ok(color) => Ok(color),
            Err(e) => Err(ColorConversionError{ message: format!("Failed to convert '{}' to a color: {:?}", c, e) }),
        }
    } else {
        Ok(default)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Theme {
    pub(crate) title: Color,
    pub(crate) text: Color,
    pub(crate) links: Color,
    pub(crate) modes: ModeColors,
    pub(crate) boundaries: Color,
    pub(crate) select: Color,
    pub(crate) search: Color,
}

#[derive(Debug, Clone)]
pub(crate) struct ModeColors {
    pub(crate) normal_mode: Color,
    pub(crate) insert_mode: Color,
    pub(crate) visual_mode: Color,
    pub(crate) search_mode: Color,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UserTheme {
    pub(crate) title: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) links: Option<String>,
    pub(crate) normal_mode: Option<String>,
    pub(crate) insert_mode: Option<String>,
    pub(crate) visual_mode: Option<String>,
    pub(crate) search_mode: Option<String>,
    pub(crate) boundaries: Option<String>,
    pub(crate) select: Option<String>,
    pub(crate) search: Option<String>,
}

impl Theme {
    fn new(
        title: Color,
        text: Color,
        links: Color,
        modes: ModeColors,
        boundaries: Color,
        select: Color,
        search: Color,
    ) -> Self {
        Self {
            title,
            text,
            links,
            modes,
            boundaries,
            select,
            search,
        }
    }

    fn default() -> Self {
        Self {
            title: Color::Red,
            text: Color::default(),
            links: Color::Blue,
            modes: ModeColors {
                normal_mode: Color::Yellow,
                insert_mode: Color::Cyan,
                visual_mode: Color::Magenta,
                search_mode: Color::Red,
            }, 
            boundaries: Color::default(),
            select: Color::Magenta,
            search: Color::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) theme: Theme,
}

impl Config {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    ColorConversion(ColorConversionError),
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::Toml(value)
    }
}

impl From<ColorConversionError> for ConfigError {
    fn from(value: ColorConversionError) -> Self {
        ConfigError::ColorConversion(value)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "IO Error: {}", err),
            ConfigError::Toml(err) => write!(f, "TOML Parsing Error: {}", err),
            ConfigError::ColorConversion(err) => write!(f, "Color Conversion Error: {}", err),
        }
    }
}

impl Error for ConfigError {}

pub(crate) fn try_load_config(path: &str) -> Result<Config, ConfigError> {
    let default_theme = Theme::default();

    if fs::metadata(path).is_ok() {
        let content = fs::read_to_string(path)?;
        let user_theme: UserTheme = toml::from_str(&content)?;
        
        let title = try_convert_color(user_theme.title, default_theme.title)?;
        let text = try_convert_color(user_theme.text, default_theme.text)?;
        let links = try_convert_color(user_theme.links, default_theme.links)?;
        let normal_mode = try_convert_color(user_theme.normal_mode, default_theme.modes.normal_mode)?;
        let insert_mode  = try_convert_color(user_theme.insert_mode, default_theme.modes.insert_mode)?;
        let visual_mode  = try_convert_color(user_theme.visual_mode, default_theme.modes.visual_mode)?;
        let search_mode = try_convert_color(user_theme.search_mode, default_theme.modes.search_mode)?;
        let boundaries = try_convert_color(user_theme.boundaries, default_theme.boundaries)?;
        let select  = try_convert_color(user_theme.select, default_theme.select)?;
        let search = try_convert_color(user_theme.search, default_theme.search)?;
        
        let modes = ModeColors {
            normal_mode,
            insert_mode,
            visual_mode,
            search_mode
        };

        let theme = Theme::new(
            title, text, links, 
            modes, boundaries, select, search
        );

        let cfg = Config::new(theme);

        Ok(cfg)
    } else {
        let default_cfg = Config::new(default_theme);
        Ok(default_cfg)
    }
}
