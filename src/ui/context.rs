use std::collections::HashMap;

use super::error::{Error, Result};

pub struct Context {
    data: HashMap<String, String>,
}
impl Context {
    pub(crate) fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn load(&self, key: impl AsRef<str>) -> Result<String> {
        let key = key.as_ref();
        self.data.get(key).cloned().ok_or_else(|| Error::LoadValue(key.into()))
    }
}
