use crate::preferences;

#[derive(Clone, PartialEq)]
pub enum Action {
    Resume,
    Quit,
    MenuHide,
    MenuShow,
    ToggleFullScreen,

    PreferencesUpdate(preferences::Preferences),

    ConsoleCommand(String),
    ConsoleHide,
    ConsoleShow,

    NewGame,
    ReplayLoad(String),
    TogglePause,
}
