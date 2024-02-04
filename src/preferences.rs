#[derive(Clone, PartialEq)]
pub enum DropIndicatorStyle {
    None,
    Outline,
    Triangles,
}

#[derive(Clone, PartialEq)]
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
