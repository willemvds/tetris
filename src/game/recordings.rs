use crate::actions;
use crate::tetrominos;

#[derive(Debug)]
enum EventKind {
    Pause,
    Unpause,
    GameOver,
    PieceSpawned(tetrominos::Kind),
    Action(actions::Action),
}

#[derive(Debug)]
struct Event {
    kind: EventKind,
    at: f64,
}

#[derive(Debug)]
pub struct Recording {
    events: Vec<Event>,
}

impl Recording {
    pub fn new() -> Recording {
        Recording { events: vec![] }
    }

    pub fn push_action(&mut self, at: f64, action: actions::Action) {
        self.events.push(Event {
            kind: EventKind::Action(action),
            at,
        })
    }

    pub fn push_piece(&mut self, at: f64, k: tetrominos::Kind) {
        self.events.push(Event {
            kind: EventKind::PieceSpawned(k),
            at,
        })
    }

    pub fn gameover(&mut self, at: f64) {
        self.events.push(Event {
            kind: EventKind::GameOver,
            at,
        })
    }
}
