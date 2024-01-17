pub struct Preferences {
    pub drop_indicator: bool,
}

impl Preferences {
    pub fn new() -> Preferences {
        Preferences {
            drop_indicator: true,
        }
    }
}
