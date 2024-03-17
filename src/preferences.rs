use serde;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum DropIndicatorStyle {
    None = 1,
    Outline = 2,
    Triangles = 3,
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
