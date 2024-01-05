use crate::actions;

#[derive(Debug)]
struct Action {
    action: actions::Action,
    at: f64,
}

#[derive(Debug)]
pub struct Recording {
    actions: Vec<Action>,
}

impl Recording {
    pub fn new() -> Recording {
        Recording { actions: vec![] }
    }

    pub fn push(&mut self, at: f64, action: actions::Action) {
        self.actions.push(Action { action, at })
    }
}
