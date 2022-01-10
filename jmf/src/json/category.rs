use crate::json::Author;
use jokotypes::*;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatDescription {
    pub name: String,
    pub display_name: String,
    pub id: CategoryID,
    pub is_separator: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatSelectionTree {
    pub id: CategoryID,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CatSelectionTree>,
}
