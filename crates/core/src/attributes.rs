use serde::{Deserialize, Serialize};

/// Descriptive metadata ("Attributes")
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Attributes {
    pub label: String,
    pub category: String,
    pub description: String,
    pub author: String,
    pub author_url: String,
    pub tags: String,
    /// A helper graph its package uses but the Library does not list
    pub hide_in_library: bool,
    pub group: String,
    pub user_data: String,
}
