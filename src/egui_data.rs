use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EGUIData {
    pub espanso_dir: String,
}
