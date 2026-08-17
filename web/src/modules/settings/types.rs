use serde::{Deserialize, Serialize};

/// Admin-configurable validation rules for the public add-request form.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RequestRules {
    pub require_birthday: bool,
    pub require_image: bool,
    pub required_info_keys: Vec<String>,
}

impl Default for RequestRules {
    fn default() -> Self {
        Self {
            require_birthday: true,
            require_image: false,
            required_info_keys: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Whether the public `/add` page and its submission endpoint are reachable.
    pub add_page_enabled: bool,
    /// Which fields the public `/add` form demands.
    pub add_request_rules: RequestRules,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            add_page_enabled: true,
            add_request_rules: RequestRules::default(),
        }
    }
}
