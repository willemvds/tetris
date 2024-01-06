use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Drop,
    MoveDown,
    MoveLeft,
    MoveRight,
    Rotate,
}
