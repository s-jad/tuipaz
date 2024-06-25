use std::{fs, num::ParseIntError, fmt::{Display, self}, error::Error, env, collections::HashMap};

use log::info;
use ratatui::style::{Color, self, Modifier};
use serde::{Deserialize, de::{MapAccess, Visitor}, Deserializer};
use tuipaz_textarea::{Input, Key};

use super::events::Action;

#[derive(Debug, Clone)]
pub(crate) struct Colors(pub HashMap<String, String>);

impl<'de> Deserialize<'de> for Colors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> 
    {
        struct ColorMapVisitor;

        impl<'de> Visitor<'de> for ColorMapVisitor {
            type Value = Colors;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting a color name and hex code")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Colors, M::Error> 
            where 
                M: MapAccess<'de>
            {
                let mut color_map = HashMap::new();

                while let Some((key, hex)) = access.next_entry::<String, String>()? {
                    color_map.insert(key, hex);
                }
                Ok(Colors(color_map))
            }
        }
        deserializer.deserialize_map(ColorMapVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TempTheme {
    pub(crate) title: String,
    pub(crate) text: String,
    pub(crate) borders: String,
    pub(crate) normal_mode: String,
    pub(crate) insert_mode: String,
    pub(crate) visual_mode: String,
    pub(crate) search_mode: String,
    pub(crate) links: String,
    pub(crate) select: String,
    pub(crate) search: String,
    pub(crate) hop: String,
    pub(crate) notelist: NoteListTheme,
}

#[derive(Debug, Clone)]
pub(crate) struct NoteListTheme {
    pub(crate) selection_modifier: Modifier,
    pub(crate) selection_symbol: String,
    pub(crate) selection_highlight: Color,
}

impl NoteListTheme {
    fn default() -> Self {
        Self {
            selection_modifier: Modifier::BOLD,
            selection_symbol: ">> ".to_string(),
            selection_highlight: Color::Magenta,
        }
    }
}

impl<'de> Deserialize<'de> for NoteListTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> 
    {
        struct NLTVisitor;

        impl<'de> Visitor<'de> for NLTVisitor {
            type Value = NoteListTheme;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting a key and note list theme value")
            }

            fn visit_map<M>(self, mut access: M) -> Result<NoteListTheme, M::Error> 
            where 
                M: MapAccess<'de>
            {
                let mut nlt = NoteListTheme::default();

                while let Some((key, hex)) = access.next_entry::<String, String>()? {
                    match key.as_str() {
                        "selection_modifier" => {
                            nlt.selection_modifier = match hex.as_str() {
                                "bold" => {
                                   Modifier::BOLD 
                                },
                                "italic" => {
                                    Modifier::ITALIC
                                },
                                "underlined" => {
                                    Modifier::UNDERLINED
                                },
                                "reversed" => {
                                    Modifier::REVERSED
                                },
                                _ => Modifier::BOLD
                            }
                        },
                        "selection_symbol" => {
                            nlt.selection_symbol = hex.to_owned()
                        },
                        "selection_highlight" => {
                            nlt.selection_highlight = 
                                try_convert_color(Some(hex), Color::Magenta).unwrap_or(Color::Magenta)
                        },
                        _ => {}
                    }
                }
                Ok(nlt)
            }
        }
        deserializer.deserialize_map(NLTVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Theme {
    pub(crate) title: Color,
    pub(crate) text: Color,
    pub(crate) modes: ModeColors,
    pub(crate) highlights: HighlightColors,
    pub(crate) borders: Color,
    pub(crate) notelist: NoteListTheme,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HighlightColors {
    pub(crate) links: Color,
    pub(crate) select: Color,
    pub(crate) search: Color,
    pub(crate) hop: Color,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ModeColors {
    pub(crate) normal_mode: Color,
    pub(crate) insert_mode: Color,
    pub(crate) visual_mode: Color,
    pub(crate) search_mode: Color,
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

fn hex_to_color(hex: &str) -> Result<style::Color, HexColorError> {
    if hex.len()!= 7 ||!hex.starts_with('#') {
        return Err(HexColorError::InvalidFormat);
    }
    let r = u8::from_str_radix(&hex[1..3], 16)?;
    let g = u8::from_str_radix(&hex[3..5], 16)?;
    let b = u8::from_str_radix(&hex[5..7], 16)?;
    Ok(style::Color::Rgb(r, g, b))
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

impl Theme {
    fn new(
        title: Color,
        text: Color,
        modes: ModeColors,
        highlights: HighlightColors,
        borders: Color,
        notelist: NoteListTheme,
    ) -> Self {
        Self {
            title,
            text,
            modes,
            highlights,
            borders,
            notelist
        }
    }

    fn default() -> Self {
        Self {
            title: Color::Red,
            text: Color::default(),
            modes: ModeColors {
                normal_mode: Color::Yellow,
                insert_mode: Color::Cyan,
                visual_mode: Color::Magenta,
                search_mode: Color::Red,
            }, 
            highlights: HighlightColors {
                links: Color::Blue,
                select: Color::Magenta,
                search: Color::Red,
                hop: Color::LightRed,
            },
            borders: Color::default(),
            notelist: NoteListTheme {
                selection_modifier: Modifier::BOLD,
                selection_symbol: ">> ".to_string(),
                selection_highlight: Color::Magenta,
            }
        }
    }
}

pub(crate) fn get_action(action: &str, input: Input) -> Action {
    match action {
        "show_exit_screen" => Action::ShowExitScreen,
        "prev_screen" => Action::PrevScreen,
        "quit" => Action::Quit,
        "save" => Action::Save,
        "load" => Action::Load,
        "delete" => Action::Delete,
        "new_note" => Action::NewNote,
        "new_title" => Action::NewTitle,
        "open_note_list" => Action::OpenNoteList,
        "toggle_searchbar" => Action::ToggleSearchbar(input),
        "toggle_sidebar" => Action::ToggleSidebar,
        "increase_sidebar" => Action::IncreaseSidebar,
        "decrease_sidebar" => Action::DecreaseSidebar,
        "switch_active_widget" => Action::SwitchActiveWidget,
        "load_selected_note" => Action::LoadSelectedNote,
        _ => Action::Null,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KeyMap(HashMap<Input, Action>);

impl<'de> Deserialize<'de> for KeyMap {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyMapVisitor;

        impl<'de> Visitor<'de> for KeyMapVisitor {
            type Value = KeyMap;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting an action and a input")
            }

            fn visit_map<M>(self, mut access: M) -> Result<KeyMap, M::Error> 
            where 
                M: MapAccess<'de>
            {
                let mut key_map = HashMap::new();

                while let Some((action, input)) = access.next_entry::<String, String>()? {
                    let mut key = Key::Null;
                    let mut ctrl = false;
                    let mut alt = false;
                    let mut shift = false;
                    info!("deserializer::input: {}", input);
                    info!("deserializer::action: {}", action);
                    if input.contains('-') {
                        let mut parts = input.split('-');
                        let modifier_part = parts.next().ok_or_else(|| serde::de::Error::custom("Modifier part missing"))?;
                        let key_part = parts.next().ok_or_else(|| serde::de::Error::custom("Key part missing"))?;
                        info!("deserializer::modifier_part: {:?}", modifier_part);
                        info!("deserializer::key_part: {:?}", key_part);
                        
                        key = match key_part.chars().next() {
                            Some(c) => Key::Char(c),
                            None => return Err(serde::de::Error::custom("Invalid key")),
                        };

                        ctrl = modifier_part.contains("ctrl");
                        alt = modifier_part.contains("alt");
                        shift = modifier_part.contains("shift");
                    } else {
                        match input.len() > 1 {
                            true => {
                                key = match input.as_str() {
                                    "esc" => Key::Esc,
                                    "tab" => Key::Tab,
                                    "enter" => Key::Enter,
                                    _ => Key::Null,
                                };
                            },
                            false => {
                                key = match input.chars().next() {
                                    Some(c) => Key::Char(c),
                                    None => return Err(serde::de::Error::custom("Invalid key")),
                                };
                            },
                        }
                    }                     

                    info!("deserializer::key: {:?}\n, ctrl: {}\n, alt: {}\n, shift: {}\n", key, ctrl, alt, shift);
                    let i = Input { key, ctrl, alt, shift };
                    let a = get_action(&action, i);
                    key_map.insert(i, a);
                }
                Ok(KeyMap(key_map))
            }
        }
        deserializer.deserialize_map(KeyMapVisitor)
    }
}


#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TempConfig {
    pub(crate) colors: Colors,
    pub(crate) theme: TempTheme,
    pub(crate) keymap: KeyMap,
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) theme: Theme,
    pub(crate) keymap: HashMap<Input, Action>,
}

impl Config {
    fn new(temp_config: TempConfig) -> Result<Self, ConfigError> {
        let keymap = temp_config.keymap.0.clone();
        let theme = get_theme(temp_config)?;

        Ok(Self {
            theme,
            keymap,
        })
    }

    fn default() -> Self {
        let theme = Theme::default();
        let keymap = HashMap::new();

        Config {
            theme,
            keymap,
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

fn get_theme(temp_config: TempConfig) -> Result<Theme, ConfigError> {
    let default_theme = Theme::default();

    let title = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.title).cloned(),
        default_theme.title
    )?;
    let text = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.text).cloned(),
        default_theme.text
    )?;
    let normal_mode = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.normal_mode).cloned(), 
        default_theme.modes.normal_mode
    )?;
    let insert_mode  = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.insert_mode).cloned(),
        default_theme.modes.insert_mode
    )?;
    let visual_mode  = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.visual_mode).cloned(),
        default_theme.modes.visual_mode
    )?;
    let search_mode = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.search_mode).cloned(),
        default_theme.modes.search_mode
    )?;
    let links = try_convert_color
        (temp_config.colors.0.get(&temp_config.theme.links).cloned(),
        default_theme.highlights.links
    )?;
    let select = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.select).cloned(),
        default_theme.highlights.select
    )?;
    let search = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.search).cloned(),
        default_theme.highlights.search
    )?;
    let hop = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.hop).cloned(),
        default_theme.highlights.hop
    )?;
    let borders = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.borders).cloned(),
        default_theme.borders
    )?;
    
    let highlights = HighlightColors {
        links,
        select,
        search,
        hop,
    };
    let modes = ModeColors {
        normal_mode,
        insert_mode,
        visual_mode,
        search_mode
    };

    Ok(Theme::new(
        title,
        text, 
        modes,
        highlights,
        borders,
        temp_config.theme.notelist,
    ))
}

pub(crate) fn try_load_config() -> Result<Config, ConfigError> {
    let path = env::current_dir().unwrap();
    let config_path = path.join("config.toml");
    let metadata = fs::metadata(&config_path);
    
    // Sanity check - metadata means file exists
    if metadata.is_ok() {
        let content = fs::read_to_string(config_path)?;
        let temp_cfg: TempConfig = toml::de::from_str(&content)?;
        info!("temp_cfg: {:?}", temp_cfg);
        let cfg = Config::new(temp_cfg);
        info!("cfg: {:?}", cfg);
        cfg
    // If no config file ... use defaults
    } else {
        let default_cfg = Config::default();
        info!("default_cfg: {:?}", default_cfg);
        Ok(default_cfg)
    }
}
