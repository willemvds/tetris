use crate::preferences;

#[derive(Clone, PartialEq)]
pub enum Action {
    ToggleFullScreen,
    ConsoleCommand(String),
    ConsoleHide,
    ConsoleShow,
    GameNew,
    MenuHide,
    MenuShow,
    PreferencesUpdate(preferences::Preferences),
    Quit,
    ReplayLoad(String),
    Resume,
    TogglePause,
}
