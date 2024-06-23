use std::fs;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub(crate) struct ColorScheme {
    pub(crate) title: String,
    pub(crate) text: String,
    pub(crate) links: String,
    pub(crate) normal_mode: String,
    pub(crate) insert_mode: String,
    pub(crate) visual_mode: String,
    pub(crate) search_mode: String,
    pub(crate) boundaries: String,
    pub(crate) select: String,
    pub(crate) search: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UserColorScheme {
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

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) colors: ColorScheme,
}

impl Config {
    fn new(colors: ColorScheme) -> Self {
        Self {
            colors,
        }
    }
}

impl ColorScheme {
    fn new(
        title: String,
        text: String,
        links: String,
        normal_mode: String,
        insert_mode: String,
        visual_mode: String,
        search_mode: String,
        boundaries: String,
        select: String,
        search: String,
    ) -> Self {
        Self {
            title,
            text,
            links,
            normal_mode,
            insert_mode,
            visual_mode,
            search_mode,
            boundaries,
            select,
            search,
        }
    }

    fn default() -> Self {
        Self {
            title: "purple".to_string(),
            text:  "default".to_string(),
            links: "blue".to_string(),
            normal_mode: "yellow".to_string(),
            insert_mode: "cyan".to_string(),
            visual_mode: "purple".to_string(),
            search_mode: "red".to_string(),
            boundaries: "default".to_string(),
            select: "purple".to_string(),
            search: "red".to_string(),
        }
    }
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let default_cs = ColorScheme::default();

    if fs::metadata(path).is_ok() {
        let content = fs::read_to_string(path)?;
        let user_cs: UserColorScheme = toml::from_str(&content)?;
        
        let cs = ColorScheme::new(
            user_cs.title.unwrap_or(default_cs.title), 
            user_cs.text.unwrap_or(default_cs.text),
            user_cs.links.unwrap_or(default_cs.links),
            user_cs.normal_mode.unwrap_or(default_cs.normal_mode),
            user_cs.insert_mode.unwrap_or(default_cs.insert_mode),
            user_cs.visual_mode.unwrap_or(default_cs.visual_mode),
            user_cs.search_mode.unwrap_or(default_cs.search_mode),
            user_cs.boundaries.unwrap_or(default_cs.boundaries),
            user_cs.select.unwrap_or(default_cs.select),
            user_cs.search.unwrap_or(default_cs.search),
        );

        let cfg = Config::new(cs);

        Ok(cfg)
    } else {
        let default_cfg = Config::new(default_cs);
        Ok(default_cfg)
    }
}
