use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt::{self, Display},
    fs,
    num::ParseIntError,
};

use log::info;
use ratatui::style::{self, Color, Modifier};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use tuipaz_textarea::{Input, Key};

use super::events::Action;

pub(crate) fn get_action(action: &str, input: Input) -> Action {
    match action {
        "show_exit_screen" => Action::ShowExitScreen,
        "prev_screen" => Action::Esc,
        "quit" => Action::Quit,
        "save" => Action::SaveNote,
        "load" => Action::LoadNote,
        "delete" => Action::DeleteNote,
        "new_note" => Action::NewNote,
        "new_title" => Action::NewTitle,
        "open_note_list" => Action::OpenNoteList,
        "toggle_searchbar" => Action::ToggleSearchbar(input),
        "toggle_sidebar" => Action::ToggleSidebar,
        "increase_sidebar" => Action::IncreaseSidebar,
        "decrease_sidebar" => Action::DecreaseSidebar,
        "switch_active_widget" => Action::SwitchActiveWidget,
        _ => Action::Null,
    }
}

fn complete_keymap(keymap: &mut HashMap<Action, Input>) {
    let default_bindings = KeyMap::get_defaults();

    for (a, i) in default_bindings.into_iter() {
        if keymap.get(&a).is_none() {
            keymap.insert(a, i);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KeyMap(HashMap<Action, Input>);

impl KeyMap {
    fn get_defaults() -> Vec<(Action, Input)> {
        vec![
            (
                Action::ShowExitScreen,
                Input {
                    key: Key::Char('q'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::Esc,
                Input {
                    key: Key::Esc,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::SaveNote,
                Input {
                    key: Key::Char('s'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::LoadNote,
                Input {
                    key: Key::Char('l'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::DeleteNote,
                Input {
                    key: Key::Char('d'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::NewNote,
                Input {
                    key: Key::Char('n'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::NewTitle,
                Input {
                    key: Key::Char('t'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::ToggleSidebar,
                Input {
                    key: Key::Char('f'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::IncreaseSidebar,
                Input {
                    key: Key::Char('.'),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::DecreaseSidebar,
                Input {
                    key: Key::Char(','),
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::Tab,
                Input {
                    key: Key::Tab,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::SwitchActiveWidget,
                Input {
                    key: Key::Tab,
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::Confirm,
                Input {
                    key: Key::Char('y'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::Cancel,
                Input {
                    key: Key::Char('n'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::DeleteChar,
                Input {
                    key: Key::Backspace,
                    ctrl: false,
                    alt: true,
                    shift: false,
                },
            ),
            (
                Action::Activate(Input {
                    key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                Input {
                    key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::Up(Input {
                    key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                Input {
                    key: Key::Up,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::Down(Input {
                    key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                Input {
                    key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::ToggleSearchbar(Input {
                    key: Key::Char('/'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                Input {
                    key: Key::Char('/'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
            (
                Action::InsertLink(Input {
                    key: Key::Char(']'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                Input {
                    key: Key::Char(']'),
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
            ),
        ]
    }
}

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
                M: MapAccess<'de>,
            {
                let mut key_map = HashMap::new();

                while let Some((action, input)) = access.next_entry::<String, String>()? {
                    let mut key = Key::Null;
                    let mut ctrl = false;
                    let mut alt = false;
                    let mut shift = false;

                    if input.contains('-') {
                        let mut parts = input.split('-');
                        let modifier_part = parts
                            .next()
                            .ok_or_else(|| serde::de::Error::custom("Modifier part missing"))?;
                        let key_part = parts
                            .next()
                            .ok_or_else(|| serde::de::Error::custom("Key part missing"))?;

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
                            }
                            false => {
                                key = match input.chars().next() {
                                    Some(c) => Key::Char(c),
                                    None => return Err(serde::de::Error::custom("Invalid key")),
                                };
                            }
                        }
                    }

                    let i = Input {
                        key,
                        ctrl,
                        alt,
                        shift,
                    };
                    let a = get_action(&action, i);
                    key_map.insert(a, i);
                }
                Ok(KeyMap(key_map))
            }
        }
        deserializer.deserialize_map(KeyMapVisitor)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Colors(pub HashMap<String, String>);

impl<'de> Deserialize<'de> for Colors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ColorMapVisitor;

        impl<'de> Visitor<'de> for ColorMapVisitor {
            type Value = Colors;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting a color name and hex code")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Colors, M::Error>
            where
                M: MapAccess<'de>,
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

#[derive(Debug, Clone)]
pub(crate) struct HeadingsTheme {
    pub(crate) main_color: Color,
    pub(crate) main_modifiers: Vec<Modifier>,
    pub(crate) sub_color: Color,
    pub(crate) sub_modifiers: Vec<Modifier>,
}

impl HeadingsTheme {
    fn default() -> Self {
        Self {
            main_color: Color::Green,
            main_modifiers: vec![Modifier::BOLD, Modifier::UNDERLINED],
            sub_color: Color::Magenta,
            sub_modifiers: vec![Modifier::ITALIC],
        }
    }
}

impl<'de> Deserialize<'de> for HeadingsTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HTVisitor;

        impl<'de> Visitor<'de> for HTVisitor {
            type Value = HeadingsTheme;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting a key and headings theme value")
            }

            fn visit_map<M>(self, mut access: M) -> Result<HeadingsTheme, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut ht = HeadingsTheme::default();
                let mut main_modifiers = vec![];
                let mut sub_modifiers = vec![];

                while let Some((key, value)) = access.next_entry::<String, String>()? {
                    match key.as_str() {
                        "main_color" => {
                            let mc = try_convert_color(Some(value), Color::Green)
                                .unwrap_or(Color::Green);
                            ht.main_color = mc;
                        }
                        "sub_color" => {
                            let sc = try_convert_color(Some(value), Color::Magenta)
                                .unwrap_or(Color::Magenta);
                            ht.sub_color = sc;
                        }
                        "main_bold" => {
                            if value == "true" {
                                main_modifiers.push(Modifier::BOLD);
                            }
                        }
                        "main_italic" => {
                            if value == "true" {
                                main_modifiers.push(Modifier::ITALIC);
                            }
                        }
                        "main_underlined" => {
                            if value == "true" {
                                main_modifiers.push(Modifier::UNDERLINED);
                            }
                        }
                        "sub_bold" => {
                            if value == "true" {
                                sub_modifiers.push(Modifier::BOLD);
                            }
                        }
                        "sub_italic" => {
                            if value == "true" {
                                sub_modifiers.push(Modifier::ITALIC);
                            }
                        }
                        "sub_underlined" => {
                            if value == "true" {
                                sub_modifiers.push(Modifier::UNDERLINED);
                            }
                        }
                        _ => {}
                    }
                }
                ht.main_modifiers = main_modifiers;
                ht.sub_modifiers = sub_modifiers;
                Ok(ht)
            }
        }
        deserializer.deserialize_map(HTVisitor)
    }
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
        D: serde::Deserializer<'de>,
    {
        struct NLTVisitor;

        impl<'de> Visitor<'de> for NLTVisitor {
            type Value = NoteListTheme;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Expecting a key and note list theme value")
            }

            fn visit_map<M>(self, mut access: M) -> Result<NoteListTheme, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut nlt = NoteListTheme::default();

                while let Some((key, hex)) = access.next_entry::<String, String>()? {
                    match key.as_str() {
                        "selection_modifier" => {
                            nlt.selection_modifier = match hex.as_str() {
                                "bold" => Modifier::BOLD,
                                "italic" => Modifier::ITALIC,
                                "underlined" => Modifier::UNDERLINED,
                                "reversed" => Modifier::REVERSED,
                                _ => Modifier::BOLD,
                            }
                        }
                        "selection_symbol" => nlt.selection_symbol = hex.to_owned(),
                        "selection_highlight" => {
                            nlt.selection_highlight = try_convert_color(Some(hex), Color::Magenta)
                                .unwrap_or(Color::Magenta)
                        }
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
pub(crate) struct ModeTheme {
    pub(crate) normal_mode: String,
    pub(crate) insert_mode: String,
    pub(crate) visual_mode: String,
    pub(crate) visual_line_mode: String,
    pub(crate) search_mode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HighlightTheme {
    pub(crate) links: String,
    pub(crate) select: String,
    pub(crate) search: String,
    pub(crate) hop: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TempTheme {
    pub(crate) note_title: String,
    pub(crate) text: String,
    pub(crate) borders: String,
    pub(crate) modes: ModeTheme,
    pub(crate) highlights: HighlightTheme,
    pub(crate) notelist: NoteListTheme,
    pub(crate) headings: HeadingsTheme,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Theme {
    pub(crate) note_title: Color,
    pub(crate) text: Color,
    pub(crate) borders: Color,
    pub(crate) modes: ModeColors,
    pub(crate) highlights: HighlightColors,
    pub(crate) notelist: NoteListTheme,
    pub(crate) headings: HeadingsTheme,
}

impl Theme {
    fn new(
        note_title: Color,
        text: Color,
        borders: Color,
        modes: ModeColors,
        highlights: HighlightColors,
        notelist: NoteListTheme,
        headings: HeadingsTheme,
    ) -> Self {
        Self {
            note_title,
            text,
            borders,
            modes,
            highlights,
            notelist,
            headings,
        }
    }

    fn default() -> Self {
        Self {
            note_title: Color::Red,
            text: Color::default(),
            borders: Color::default(),
            modes: ModeColors {
                normal_mode: Color::Yellow,
                insert_mode: Color::Cyan,
                visual_mode: Color::Magenta,
                visual_line_mode: Color::LightMagenta,
                search_mode: Color::Red,
            },
            highlights: HighlightColors {
                links: Color::Blue,
                select: Color::Magenta,
                search: Color::Red,
                hop: Color::LightRed,
            },
            notelist: NoteListTheme {
                selection_modifier: Modifier::BOLD,
                selection_symbol: ">> ".to_string(),
                selection_highlight: Color::Magenta,
            },
            headings: HeadingsTheme {
                main_color: Color::Green,
                main_modifiers: vec![Modifier::BOLD, Modifier::UNDERLINED],
                sub_color: Color::Magenta,
                sub_modifiers: vec![Modifier::ITALIC],
            },
        }
    }
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
    pub(crate) visual_line_mode: Color,
    pub(crate) search_mode: Color,
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
    pub(crate) keymap: HashMap<Action, Input>,
}

impl Config {
    fn new(temp_config: TempConfig) -> Result<Self, ConfigError> {
        let mut keymap = temp_config.keymap.0.clone();
        let theme = get_theme(temp_config)?;
        complete_keymap(&mut keymap);

        Ok(Self { theme, keymap })
    }

    fn default() -> Self {
        let theme = Theme::default();
        let mut keymap = HashMap::new();
        complete_keymap(&mut keymap);

        Config { theme, keymap }
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
    if hex.len() != 7 || !hex.starts_with('#') {
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

fn try_convert_color(
    user_color: Option<String>,
    default: Color,
) -> Result<Color, ColorConversionError> {
    if let Some(c) = user_color {
        match hex_to_color(&c) {
            Ok(color) => Ok(color),
            Err(e) => Err(ColorConversionError {
                message: format!("Failed to convert '{}' to a color: {:?}", c, e),
            }),
        }
    } else {
        Ok(default)
    }
}

macro_rules! convert_color {
    ($config:expr, $theme_path:expr, $default_option:expr) => {{
        let color_option = $config.colors.0.get($theme_path).cloned();

        try_convert_color(color_option, $default_option)
    }};
}

fn get_theme(temp_config: TempConfig) -> Result<Theme, ConfigError> {
    let default_theme = Theme::default();

    let title = convert_color!(
        temp_config,
        &temp_config.theme.note_title,
        default_theme.note_title
    )?;
    let text = try_convert_color(
        temp_config.colors.0.get(&temp_config.theme.text).cloned(),
        default_theme.text,
    )?;
    let normal_mode = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.modes.normal_mode)
            .cloned(),
        default_theme.modes.normal_mode,
    )?;
    let insert_mode = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.modes.insert_mode)
            .cloned(),
        default_theme.modes.insert_mode,
    )?;
    let visual_mode = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.modes.visual_mode)
            .cloned(),
        default_theme.modes.visual_mode,
    )?;
    let visual_line_mode = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.modes.visual_line_mode)
            .cloned(),
        default_theme.modes.visual_line_mode,
    )?;
    let search_mode = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.modes.search_mode)
            .cloned(),
        default_theme.modes.search_mode,
    )?;
    let links = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.highlights.links)
            .cloned(),
        default_theme.highlights.links,
    )?;
    let select = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.highlights.select)
            .cloned(),
        default_theme.highlights.select,
    )?;
    let search = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.highlights.search)
            .cloned(),
        default_theme.highlights.search,
    )?;
    let hop = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.highlights.hop)
            .cloned(),
        default_theme.highlights.hop,
    )?;
    let borders = try_convert_color(
        temp_config
            .colors
            .0
            .get(&temp_config.theme.borders)
            .cloned(),
        default_theme.borders,
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
        visual_line_mode,
        search_mode,
    };

    Ok(Theme::new(
        title,
        text,
        borders,
        modes,
        highlights,
        temp_config.theme.notelist,
        temp_config.theme.headings,
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
