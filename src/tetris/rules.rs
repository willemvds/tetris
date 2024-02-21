use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub enum ScoringSystemKind {
    OriginalBPS,
    OriginalSega,
    OriginalNintendo,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Rules {
    pub lock_delay: u32,
    pub lock_delay_on_hard_drop: bool,
    pub wall_kicks: bool,
    pub scoring_system: ScoringSystemKind,
    pub action_cooldown: u8,
}

impl Rules {
    pub fn new() -> Rules {
        Rules {
            lock_delay: 0,
            lock_delay_on_hard_drop: false,
            wall_kicks: true,
            scoring_system: ScoringSystemKind::OriginalBPS,
            action_cooldown: 20, // 20 ticks = 80ms cooldown
        }
    }

    pub fn lock_delay(&mut self, ld: u32) {
        self.lock_delay = ld
    }

    pub fn lock_delay_on_hard_drop(&mut self, v: bool) {
        self.lock_delay_on_hard_drop = v
    }

    pub fn scoring_system(&mut self, kind: ScoringSystemKind) {
        self.scoring_system = kind
    }
}
