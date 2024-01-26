use crate::tetris;
use crate::tetris::game;
use crate::tetris::tetrominos;

use serde::{Deserialize, Serialize};

pub struct Replay {
    pub recording: tetris::recordings::Recording,
}

#[derive(Serialize, Deserialize)]
pub struct ReplayPieces {
    pieces: Vec<tetrominos::Kind>,
    idx: usize,
}

impl ReplayPieces {
    pub fn new(replay: &Replay) -> ReplayPieces {
        let piece_events = replay
            .recording
            .events
            .iter()
            .filter(|ev| matches!(ev.kind, tetris::recordings::EventKind::PieceSpawned(_)));
        let pieces = piece_events
            .map(|ev| {
                if let tetris::recordings::EventKind::PieceSpawned(k) = ev.kind {
                    k
                } else {
                    panic!("BAD")
                }
            })
            .collect();
        ReplayPieces { pieces, idx: 0 }
    }
}

#[typetag::serde]
impl game::PieceProvider for ReplayPieces {
    fn next(&mut self) -> Result<tetrominos::Kind, String> {
        if self.idx >= self.pieces.len() {
            return Err("NO PIECES LEFT IN THE REPLAY".to_string());
        }

        let p = self.pieces[self.idx];

        self.idx += 1;

        Ok(p)
    }
}
