use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct YamlPairs {
    pub trigger: String,
    pub replace: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct EspansoYaml {
    // pub matches: Vec<HashMap<String, String>>,
    pub matches: Vec<YamlPairs>,
}
