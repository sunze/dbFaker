// --- 数据结构 ---

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Dataset {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(PartialEq)]
pub enum AppTab {
    Database,
    Dataset,
}

pub enum Mode {
    Normal,
    Creating,
    Preview(Vec<String>),
}