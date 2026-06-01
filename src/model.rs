#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    List,
    Summary,
    TypeSection(String),
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryParams {
    pub help_requested: bool,
    pub display_mode: DisplayMode,
    pub files_query: Option<String>,
    pub since: Option<String>,
    pub between: Option<(String, String)>,
    pub filter_terms: Option<Vec<String>>,
    pub latest: Option<usize>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            help_requested: false,
            display_mode: DisplayMode::List,
            files_query: None,
            since: None,
            between: None,
            filter_terms: None,
            latest: None,
        }
    }
}
