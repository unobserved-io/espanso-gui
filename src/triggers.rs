use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Triggers {
    matches: Vec<HashMap<String, String>>,
}
