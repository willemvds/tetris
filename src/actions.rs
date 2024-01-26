use crate::tetris;

#[derive(PartialEq)]
pub enum Action {
    Play,
    Quit,
    MenuHide,
    MenuShow,

    ConsoleCommand(String),
    ConsoleHide,
    ConsoleShow,

    NewGame,
    QueueGameAction(tetris::actions::Action),
    TogglePause,
}
