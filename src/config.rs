
use std::collections::HashMap;

pub enum ReplaceItem {
    Word(String),
    Handler(String),
}

pub struct ZmKeyword {
    pub name: String,
    pub help: String,
    pub mapping: Option<HashMap<String, String>>,
    pub default: Option<ReplaceItem>,
    pub prefix: Option<String>,
}



