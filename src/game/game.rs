use crate::game::recordings;

use crate::actions;
use crate::playfield;
use crate::tetrominos;

use rand;
use rand::Rng;
use serde::{Deserialize, Serialize};
use typetag;

#[derive(Clone, Serialize, Deserialize)]
pub struct Rules {
    lock_delay: u32,
}

impl Rules {
    pub fn new() -> Rules {
        Rules { lock_delay: 50 }
    }
}

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
        return &t.forms[self.rotation as usize];
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
        if self.pieces.len() == 0 {
            self.pieces = TetrominoBag::one_of_each_kind();
        }

        let mut rng = rand::thread_rng();
        let n1: usize = rng.gen_range(0..self.pieces.len());

        Ok(self.pieces.swap_remove(n1))
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
enum State {
    Init,
    Playing,
    GameOver,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    rules: Rules,
    state: State,
    ticks: usize,
    // speed is measured in number of ticks so that:
    // 1) A lower value means fewer ticks and thus faster.
    // 2) 1 is the lowest value which means on every tick.
    // 3) A higher value means more ticks and thus slower.
    pub speed: u8,
    pub play_field: playfield::PlayField,
    pub next_piece: tetrominos::Kind,
    piece_provider: Box<dyn PieceProvider>,
    pub piece: Piece,
    pub score_points: u32,
    pub score_lines_cleared: u32,
    next_action: Option<actions::Action>,

    pub recording: recordings::Recording,
}

impl Game {
    pub fn new(
        rules: Rules,
        piece_provider: Option<Box<dyn PieceProvider>>,
    ) -> Result<Game, String> {
        let play_field = playfield::PlayField::new(22, 10)?;

        let provider = match piece_provider {
            Some(p) => p,
            None => Box::new(TetrominoBag::new()),
        };

        let mut g = Game {
            rules,
            state: State::Init,
            ticks: 0,
            speed: 40,
            play_field,

            piece_provider: provider,
            piece: Piece::new(tetrominos::Kind::Stick),
            next_piece: tetrominos::Kind::Stick,

            score_points: 0,
            score_lines_cleared: 0,

            next_action: None,
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

    pub fn tick(&mut self, t: f64) -> usize {
        if self.state == State::GameOver {
            return self.ticks;
        }

        self.ticks += 1;
        //println!("SIMULATING GAME ENGINE... {:?} {:?} {:?}", t, dt, acc);
        //
        if self.next_action.is_some() {
            let action = self.next_action.unwrap();
            self.recording.push_action(self.ticks, action);

            if self.piece.busy_locking {
                self.piece.remaining_lock_frames = self.rules.lock_delay;
            }

            match self.next_action {
                Some(actions::Action::MoveDown) => self.drop_one(),
                Some(actions::Action::MoveLeft) => self.move_left(),
                Some(actions::Action::MoveRight) => self.move_right(),
                Some(actions::Action::Rotate) => self.rotate(),
                Some(actions::Action::Drop) => self.drop_fast(),
                _ => (),
            }

            self.next_action = None;
        }

        self.piece.creep += 1;
        if self.piece.creep > self.speed as usize {
            // move the piece
            self.piece.creep = 0;

            if self.can_fall() {
                self.piece.y += 1;
            } else {
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
                self.piece.remaining_lock_frames = self.rules.lock_delay;
            }
        }

        let lines_cleared = self.play_field.clear_full_rows();
        self.score_lines_cleared += lines_cleared;
        self.score_points += lines_cleared * 10;
        self.play_field.collapse();

        self.ticks
    }

    pub fn is_gameover(&self) -> bool {
        return self.state == State::GameOver;
    }

    pub fn queue_action(&mut self, a: actions::Action) -> Result<(), String> {
        if self.state != State::Playing {
            return Err("Can't queue action while game is not ready".to_string());
        }

        if self.next_action.is_none() {
            self.next_action = Some(a);

            return Ok(());
        }

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
        }
    }

    pub fn move_right(&mut self) {
        if !self.play_field.has_collission(
            self.piece.y as usize,
            self.piece.x as usize + 1,
            self.piece.form(),
        ) {
            self.piece.x += 1;
        }
    }

    fn can_fall(&mut self) -> bool {
        return !self.play_field.has_collission(
            self.piece.y as usize + 1,
            self.piece.x as usize,
            self.piece.form(),
        );
    }

    pub fn drop_distance(&mut self) -> usize {
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

    pub fn drop_fast(&mut self) {
        while self.can_fall() {
            self.piece.y += 1;
        }
        self.piece.creep = self.speed as usize
    }

    pub fn speed_up(&mut self) {
        self.speed -= 4;
        println!("NEW SPEED is {}", self.speed);
    }

    pub fn speed_down(&mut self) {
        self.speed += 4;
        println!("NEW SPEED is {}", self.speed);
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

fn new_tetromino_bag() -> Vec<&'static tetrominos::Tetromino> {
    return vec![
        tetrominos::from_kind(tetrominos::Kind::Stick),
        tetrominos::from_kind(tetrominos::Kind::Square),
        tetrominos::from_kind(tetrominos::Kind::Pyramid),
        tetrominos::from_kind(tetrominos::Kind::Seven),
        tetrominos::from_kind(tetrominos::Kind::Snake),
        tetrominos::from_kind(tetrominos::Kind::Hook),
        tetrominos::from_kind(tetrominos::Kind::Zig),
    ];
}
