#[derive(Clone, PartialEq)]
pub enum Action {
    Play,
    Quit,
    MenuHide,
    MenuShow,
    ToggleFullScreen,

    ConsoleCommand(String),
    ConsoleHide,
    ConsoleShow,

    NewGame,
    TogglePause,
}
