use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Whether the public `/add` page and its submission endpoint are reachable.
    pub add_page_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            add_page_enabled: true,
        }
    }
}
