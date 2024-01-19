#[derive(PartialEq)]
pub enum Action {
    Quit,
    MenuHide,
    ConsoleCommand(String),
    ConsoleHide,
    ConsoleShow,
}
