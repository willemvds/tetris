use crate::actions;
use crate::tetrominos;

#[derive(Debug)]
enum EventKind {
    GameOver,
    PieceSpawned(tetrominos::Kind),
}

#[derive(Debug)]
struct Event {
    kind: EventKind,
    at: f64,
}

#[derive(Debug)]
struct Action {
    action: actions::Action,
    at: f64,
}

#[derive(Debug)]
pub struct Recording {
    actions: Vec<Action>,
    events: Vec<Event>,
}

impl Recording {
    pub fn new() -> Recording {
        Recording {
            actions: vec![],
            events: vec![],
        }
    }

    pub fn push(&mut self, at: f64, action: actions::Action) {
        self.actions.push(Action { action, at })
    }

    pub fn new_piece(&mut self, at: f64, k: tetrominos::Kind) {
        self.events.push(Event {
            kind: EventKind::PieceSpawned(k),
            at,
        })
    }
}
