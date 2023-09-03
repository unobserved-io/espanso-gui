use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Triggers {
    matches: Vec<HashMap<String, String>>,
}

impl Triggers {
    fn new() -> Self {}
}
