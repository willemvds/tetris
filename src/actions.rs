#[derive(PartialEq)]
pub enum Action {
    Quit,
    HideConsole,
    HideMenu,
    ConsoleCommand(String),
}
