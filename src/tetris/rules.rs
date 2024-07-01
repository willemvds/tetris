use crate::tetris::scoring;

use serde;

#[repr(u8)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ActionCooldown {
    Shared(u8),
    Each(u8),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Rules {
    pub lock_delay: u32,
    pub lock_delay_on_hard_drop: bool,
    pub wall_kicks: bool,
    pub scoring_system: scoring::Kind,
    pub action_cooldown: ActionCooldown,
}

impl Rules {
    pub fn new() -> Rules {
        Rules {
            lock_delay: 0,
            lock_delay_on_hard_drop: false,
            wall_kicks: true,
            scoring_system: scoring::Kind::OriginalBPS,
            action_cooldown: ActionCooldown::Shared(20), // 20 ticks = 80ms cooldown
        }
    }

    pub fn lock_delay(&mut self, ld: u32) {
        self.lock_delay = ld
    }

    pub fn lock_delay_on_hard_drop(&mut self, v: bool) {
        self.lock_delay_on_hard_drop = v
    }

    pub fn scoring_system(&mut self, kind: scoring::Kind) {
        self.scoring_system = kind
    }

    pub fn action_cooldown(&mut self, action_cooldown: ActionCooldown) {
        self.action_cooldown = action_cooldown
    }
}
