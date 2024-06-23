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

#[derive(Debug, Clone)]
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

impl ColorScheme {
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
