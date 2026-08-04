use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum EditField<T> {
    #[default]
    Unchanged,
    Changed(T),
    Delete,
}
