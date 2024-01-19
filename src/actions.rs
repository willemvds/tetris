#[derive(PartialEq)]
pub enum Action {
    Quit,
    HideConsole,
    ConsoleCommand(String),
}
