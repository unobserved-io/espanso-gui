// espansoGUI - GUI to interface with Espanso
// Copyright (C) 2023 Ricky Kresslein <ricky@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
