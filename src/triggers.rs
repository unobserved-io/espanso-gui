use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Triggers {
    pub matches: Vec<HashMap<String, String>>,
}
