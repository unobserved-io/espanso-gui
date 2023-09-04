use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Triggers {
    pub matches: Vec<HashMap<String, String>>,
}
