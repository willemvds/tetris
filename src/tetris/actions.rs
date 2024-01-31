use serde::{Deserialize, Serialize};

#[derive(Eq, Hash, Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[repr(u8)]
pub enum Action {
    Drop,
    MoveDown,
    MoveLeft,
    MoveRight,
    Rotate,
}
