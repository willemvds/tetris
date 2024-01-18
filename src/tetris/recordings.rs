use crate::tetris::actions;
use crate::tetris::tetrominos;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum EventKind {
    Pause,
    Unpause,
    GameOver,
    PieceSpawned(tetrominos::Kind),
    Action(actions::Action),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub kind: EventKind,
    pub at: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recording {
    pub events: Vec<Event>,
}

impl Recording {
    pub fn new() -> Recording {
        Recording { events: vec![] }
    }

    pub fn push_action(&mut self, at: usize, action: actions::Action) {
        self.events.push(Event {
            kind: EventKind::Action(action),
            at,
        })
    }

    pub fn push_piece(&mut self, at: usize, k: tetrominos::Kind) {
        self.events.push(Event {
            kind: EventKind::PieceSpawned(k),
            at,
        })
    }

    pub fn gameover(&mut self, at: usize) {
        self.events.push(Event {
            kind: EventKind::GameOver,
            at,
        })
    }
}
