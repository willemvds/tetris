use crate::preferences;

#[derive(Clone, PartialEq)]
pub enum Action {
    Play,
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
