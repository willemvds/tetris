use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Drop,
    MoveDown,
    MoveLeft,
    MoveRight,
    Rotate,
}
