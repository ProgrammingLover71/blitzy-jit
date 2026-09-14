use std::collections::HashMap;
use std::sync::Arc;


#[derive(Clone)]
pub struct Interner {
    strings: HashMap<String, Arc<str>>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }

    pub fn intern(&mut self, value: &str) -> Arc<str> {
        self.strings
            .entry(value.to_owned())
            .or_insert_with_key(|key| Arc::from(key.as_str()))
            .clone()
    }
}