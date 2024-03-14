use std::collections;

use crate::tetris::actions;
use crate::tetris::playfield;
use crate::tetris::recordings;
use crate::tetris::rules;
use crate::tetris::scoring;
use crate::tetris::tetrominos;

use rand;
use rand::Rng;
use serde::{Deserialize, Serialize};
use typetag;

#[derive(Serialize, Deserialize)]
pub struct Piece {
    pub tetromino: tetrominos::Kind,
    pub x: u16,
    pub y: u16,
    creep: usize,
    rotation: u8,
    busy_locking: bool,
    remaining_lock_frames: u32,
}

impl Piece {
    fn new(k: tetrominos::Kind) -> Piece {
        Piece {
            tetromino: k,
            x: 4,
            y: 0,
            creep: 0,
            rotation: 0,
            busy_locking: false,
            remaining_lock_frames: 0,
        }
    }

    pub fn form(&self) -> &tetrominos::Form {
        let t = tetrominos::from_kind(self.tetromino);

        &t.forms[self.rotation as usize]
    }
}

#[typetag::serde(tag = "type")]
pub trait PieceProvider {
    fn next(&mut self) -> Result<tetrominos::Kind, String>;
}

#[derive(Serialize, Deserialize)]
struct TetrominoBag {
    pieces: Vec<tetrominos::Kind>,
}

impl TetrominoBag {
    fn new() -> TetrominoBag {
        TetrominoBag {
            pieces: Self::one_of_each_kind(),
        }
    }

    fn one_of_each_kind() -> Vec<tetrominos::Kind> {
        vec![
            tetrominos::Kind::Stick,
            tetrominos::Kind::Square,
            tetrominos::Kind::Pyramid,
            tetrominos::Kind::Seven,
            tetrominos::Kind::Snake,
            tetrominos::Kind::Hook,
            tetrominos::Kind::Zig,
        ]
    }
}

#[typetag::serde]
impl PieceProvider for TetrominoBag {
    fn next(&mut self) -> Result<tetrominos::Kind, String> {
        if self.pieces.is_empty() {
            self.pieces = TetrominoBag::one_of_each_kind();
        }

        let mut rng = rand::thread_rng();
        let n1: usize = rng.gen_range(0..self.pieces.len());

        Ok(self.pieces.swap_remove(n1))
    }
}

struct QueuedAction {
    action: actions::Action,
    queued_at: usize,
}

#[derive(PartialEq, Serialize, Deserialize)]
enum State {
    Init,
    Playing,
    GameOver,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub rules: rules::Rules,
    state: State,
    ticks: usize,
    // speed is measured in number of ticks so that:
    // 1) A lower value means fewer ticks and thus faster.
    // 2) 1 is the lowest value which means on every tick.
    // 3) A higher value means more ticks and thus slower.
    pub level: u8,
    pub speed: u8,
    pub play_field: playfield::PlayField,
    pub next_piece: tetrominos::Kind,
    piece_provider: Box<dyn PieceProvider>,
    pub piece: Piece,
    scoring_system: Box<dyn scoring::System>,
    pub score_points: u32,
    pub score_lines_cleared: u32,
    level_lines_cleared: u32,
    next_action: Option<actions::Action>,
    last_action_at: usize,
    actions_last_used_at: collections::HashMap<actions::Action, usize>,

    pub recording: recordings::Recording,
}

pub const fn calculate_speed_from_level(level: u8) -> u8 {
    match level {
        1 => 70,
        2 => 60,
        3 => 51,
        4 => 43,
        5 => 36,
        6 => 30,
        7 => 25,
        8 => 21,
        9 => 18,
        10 => 16,
        11 => 15,
        12 => 14,
        13 => 13,
        14 => 12,
        15 => 11,
        _ => 10,
    }
}

impl Game {
    pub fn new(
        rules: rules::Rules,
        piece_provider: Option<Box<dyn PieceProvider>>,
    ) -> Result<Game, String> {
        let play_field = playfield::PlayField::new(22, 10)?;

        let provider = match piece_provider {
            Some(p) => p,
            None => Box::new(TetrominoBag::new()),
        };

        let ss: Box<dyn scoring::System> = match rules.scoring_system {
            scoring::Kind::DS => Box::new(scoring::DS::new()),
            scoring::Kind::OriginalBPS => Box::new(scoring::OriginalBPS::new()),
            scoring::Kind::OriginalNintendo => Box::new(scoring::OriginalNintendo::new()),
            scoring::Kind::OriginalSega => Box::new(scoring::OriginalSega::new()),
        };

        let mut g = Game {
            rules,
            state: State::Init,
            ticks: 0,
            level: 1,
            speed: calculate_speed_from_level(1),
            play_field,

            piece_provider: provider,
            piece: Piece::new(tetrominos::Kind::Stick),
            next_piece: tetrominos::Kind::Stick,

            scoring_system: ss,
            score_points: 0,
            score_lines_cleared: 0,
            level_lines_cleared: 0,

            next_action: None,
            last_action_at: 0,
            actions_last_used_at: collections::HashMap::from([]),
            recording: recordings::Recording::new(),
        };

        // Grab the first two pieces from the first tetromino bag
        // to replace the temp values set above.
        let _ = g.grab_next_piece();
        g.recording.push_piece(0, g.next_piece);
        let _ = g.grab_next_piece();
        g.recording.push_piece(0, g.next_piece);

        g.state = State::Playing;

        Ok(g)
    }

    pub fn tick(&mut self) -> usize {
        if self.state == State::GameOver {
            return self.ticks;
        }

        self.ticks += 1;
        //println!("SIMULATING GAME ENGINE... {:?} {:?} {:?}", t, dt, acc);
        //
        let mut dropped = false;
        if self.next_action.is_some() {
            let action = self.next_action.unwrap();
            self.recording.push_action(self.ticks, action);

            match action {
                actions::Action::MoveDown => self.drop_one(),
                actions::Action::MoveLeft => self.move_left(),
                actions::Action::MoveRight => self.move_right(),
                actions::Action::Rotate => self.rotate(),
                actions::Action::Drop => {
                    dropped = self.drop_fast();
                }
            }

            self.next_action = None;
            self.actions_last_used_at.insert(action, self.ticks);
        }

        self.piece.creep += 1;
        if self.piece.creep > self.speed as usize {
            // move the piece
            self.piece.creep = 0;

            if self.can_fall() {
                self.piece.y += 1;
            }
        }

        if !self.can_fall() {
            if self.piece.busy_locking {
                if self.piece.remaining_lock_frames == 0 {
                    self.piece.busy_locking = false;
                    self.imprint_piece();
                    if let Err(e) = self.grab_next_piece() {
                        println!("grab piece err: {}", e);
                        self.state = State::GameOver;
                        self.recording.gameover(self.ticks);

                        return self.ticks;
                    }
                    self.recording.push_piece(self.ticks, self.next_piece);
                } else {
                    self.piece.remaining_lock_frames -= 1;
                }
            } else {
                self.piece.busy_locking = true;
                if !dropped || self.rules.lock_delay_on_hard_drop {
                    self.piece.remaining_lock_frames = self.rules.lock_delay;
                };
            }
        }

        let lines_cleared = self.play_field.clear_full_rows();
        self.level_lines_cleared += lines_cleared;
        self.score_lines_cleared += lines_cleared;

        self.score_points += self
            .scoring_system
            .lines_cleared(self.level, lines_cleared as u8);
        self.play_field.collapse();

        if self.level_lines_cleared >= 4 * self.level as u32 {
            self.level += 1;
            self.level_lines_cleared = 0;
            self.speed = calculate_speed_from_level(self.level);
        }

        self.ticks
    }

    pub fn is_gameover(&self) -> bool {
        self.state == State::GameOver
    }

    pub fn queue_action(&mut self, a: actions::Action) -> Result<(), String> {
        if self.state != State::Playing {
            return Err("Can't queue action while game is not ready".to_string());
        }

        let action_last_used_at = self.actions_last_used_at.entry(a).or_insert_with(|| 0);
        if *action_last_used_at + self.rules.action_cooldown as usize >= self.ticks {
            return Err("Can't queue action while game IS ON COOLDOWN".to_string());
        }

        if self.next_action.is_none() {
            self.next_action = Some(a);

            return Ok(());
        }

        dbg!("DROPPED ACTION", a);
        Err("Already have action queued for next game tick".to_string())
    }

    pub fn rotate(&mut self) {
        let mut next_rotation = self.piece.rotation + 1;
        if next_rotation >= 4 {
            next_rotation = 0;
        }

        let next_shape = tetrominos::from_kind(self.piece.tetromino).forms[next_rotation as usize];

        if !self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize,
            &next_shape,
        ) {
            self.piece.rotation = next_rotation;
            return;
        }

        if !self.rules.wall_kicks {
            return;
        }

        // try moving piece left
        if !self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize - 1,
            &next_shape,
        ) {
            self.piece.rotation = next_rotation;
            self.piece.x -= 1;
            return;
        }

        // try moving piece right
        if !self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize + 1,
            &next_shape,
        ) {
            self.piece.rotation = next_rotation;
            self.piece.x += 1;
        }
    }

    fn reset_remaining_lock_frames(&mut self) {
        if self.piece.busy_locking {
            self.piece.remaining_lock_frames = self.rules.lock_delay;
        }
    }

    pub fn move_left(&mut self) {
        if self.piece.x == 0 {
            return;
        }

        if !self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize - 1,
            self.piece.form(),
        ) {
            self.piece.x -= 1;
            self.reset_remaining_lock_frames();
        }
    }

    pub fn move_right(&mut self) {
        if !self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize + 1,
            self.piece.form(),
        ) {
            self.piece.x += 1;
            self.reset_remaining_lock_frames();
        }
    }

    fn can_fall(&mut self) -> bool {
        return !self.play_field.has_collission(
            self.piece.y as usize + 1,
            self.piece.x as usize,
            self.piece.form(),
        );
    }

    pub fn drop_distance(&self) -> usize {
        let mut distance = 0;

        while !self.play_field.has_collission(
            self.piece.y as usize + distance,
            self.piece.x as usize,
            self.piece.form(),
        ) {
            distance += 1
        }

        distance
    }

    pub fn drop_one(&mut self) {
        self.piece.creep = self.speed as usize;
    }

    pub fn drop_fast(&mut self) -> bool {
        let mut dropped = false;
        while self.can_fall() {
            self.piece.y += 1;
            dropped = true;
            self.piece.creep = self.speed as usize;
        }

        dropped
    }

    pub fn grab_next_piece(&mut self) -> Result<(), String> {
        let next_piece = self.piece_provider.next()?;

        self.piece.tetromino = self.next_piece;
        self.next_piece = next_piece;
        self.piece.rotation = 0;
        self.piece.x = (self.play_field.well_x() + (self.play_field.cols / 2) - 2) as u16;
        self.piece.y = 2;
        self.piece.creep = 0;

        if self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize,
            self.piece.form(),
        ) {
            return Err("GAME OVER - NOT ENOUGH SPACE IN WELL TO PLACE NEXT PIECE xD".to_string());
        }

        Ok(())
    }

    fn imprint_piece(&mut self) {
        let row_offset = self.piece.y as usize;
        let col_offset = self.piece.x as usize;

        let shape = self.piece.form();

        for row in 0..4 {
            for col in 0..4 {
                if shape[row][col] == 1 {
                    self.play_field.matrix[row + row_offset][col + col_offset] =
                        playfield::Location::Filled(self.piece.tetromino);
                }
            }
        }
    }
}
