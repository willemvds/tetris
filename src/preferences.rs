#[derive(PartialEq)]
pub enum DropIndicatorStyle {
    None,
    Outline,
    Triangles,
}

pub struct Preferences {
    pub drop_indicator: DropIndicatorStyle,
}

impl Preferences {
    pub fn new() -> Preferences {
        Preferences {
            drop_indicator: DropIndicatorStyle::Triangles,
        }
    }
}
