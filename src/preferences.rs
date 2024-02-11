use serde;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DropIndicatorStyle {
    None,
    Outline,
    Triangles,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Preferences {
    pub drop_indicator: DropIndicatorStyle,
}

impl Preferences {
    pub fn new() -> Preferences {
        Preferences {
            drop_indicator: DropIndicatorStyle::Outline,
        }
    }
}
