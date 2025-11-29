use crate::utils::compat::{HashMap, String, Vec};

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub params: HashMap<String, String>,
    pub text_content: String,
    pub children: Vec<Tag>,
}

impl Tag {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            params: HashMap::new(),
            text_content: String::new(),
            children: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.name = String::new();
        self.params.clear();
        self.text_content = String::new();
        self.children.clear();
    }
}