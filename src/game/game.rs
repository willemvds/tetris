use crate::game::recordings;

use crate::actions;
use crate::playfield;
use crate::tetrominos;

use rand;
use rand::Rng;

#[derive(Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Piece {
    pub tetromino: &'static tetrominos::Tetromino,
    pub x: u16,
    pub y: u16,
    creep: f64,
    rotation: u8,
}

impl Piece {
    fn new(t: &'static tetrominos::Tetromino) -> Piece {
        Piece {
            tetromino: t,
            x: 4,
            y: 0,
            creep: 0.0,
            rotation: 0,
        }
    }

    pub fn form(&self) -> &tetrominos::Form {
        return &self.tetromino.forms[self.rotation as usize];
    }
}

#[derive(PartialEq)]
enum State {
    Init,
    Playing,
    Paused,
    GameOver,
}

pub struct Game {
    state: State,
    pub speed: f64,
    pub play_field: playfield::PlayField,
    pub next_piece: &'static tetrominos::Tetromino,
    pub piece_bag: Vec<&'static tetrominos::Tetromino>,
    pub piece: Piece,
    pub score_points: u32,
    pub score_lines_cleared: u32,
    next_action: Option<actions::Action>,

    pub recording: recordings::Recording,
}

impl Game {
    pub fn new() -> Result<Game, String> {
        let play_field = playfield::PlayField::new(24, 10)?;
        let mut g = Game {
            state: State::Init,
            speed: 42.0,
            play_field,

            piece: Piece::new(tetrominos::from_kind(tetrominos::Kind::Stick)),
            next_piece: tetrominos::from_kind(tetrominos::Kind::Stick),
            piece_bag: new_tetromino_bag(),

            score_points: 0,
            score_lines_cleared: 0,

            next_action: None,
            recording: recordings::Recording::new(),
        };

        // Grab the first two pieces from the first tetromino bag
        // to replace the temp values set above.
        let _ = g.grab_next_piece();
        let _ = g.grab_next_piece();

        g.state = State::Playing;

        Ok(g)
    }

    pub fn sim(&mut self, t: f64, dt: f64, _acc: f64) {
        //println!("SIMULATING GAME ENGINE... {:?} {:?} {:?}", t, dt, acc);
        //
        if self.next_action.is_some() {
            let action = self.next_action.unwrap();
            self.recording.push(t, action);

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

        self.piece.creep += dt;
        if self.piece.creep > dt * self.speed {
            // move the piece
            self.piece.creep = 0.0;

            let bottom = self.piece.y + 4;
            if bottom as usize == self.play_field.matrix.len() {
                // we are already on the floor so leave us and create a new piece

                // overlay the shape + position onto the map
                self.imprint_piece();

                if self.grab_next_piece().is_err() {
                    self.state = State::GameOver;
                }
            } else {
                if self.can_fall() {
                    self.piece.y += 1;
                } else {
                    self.imprint_piece();
                    if self.grab_next_piece().is_err() {
                        self.state = State::GameOver;
                    }
                }
            }
            let lines_cleared = self.play_field.clear_full_rows();
            self.score_lines_cleared += lines_cleared;
            self.score_points += lines_cleared * 10;
            self.play_field.collapse();
        }
    }

    pub fn pause(&mut self) {
        if self.state == State::Playing {
            self.state = State::Paused
        }
    }

    pub fn unpause(&mut self) {
        if self.state == State::Paused {
            self.state = State::Playing
        }
    }

    pub fn is_playing(&self) -> bool {
        return self.state == State::Playing;
    }

    pub fn is_paused(&self) -> bool {
        return self.state == State::Paused;
    }

    pub fn is_gameover(&self) -> bool {
        return self.state == State::GameOver;
    }

    pub fn queue_action(&mut self, a: actions::Action) {
        if self.state != State::Playing {
            return;
        }

        if self.next_action.is_none() {
            self.next_action = Some(a)
        }
    }

    pub fn rotate(&mut self) {
        let mut next_rotation = self.piece.rotation + 1;
        if next_rotation >= 4 {
            next_rotation = 0;
        }
        let next_shape = self.piece.tetromino.forms[next_rotation as usize];

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
        if self.can_fall() {
            self.piece.y += 1;
        }
    }

    pub fn drop_fast(&mut self) {
        while self.can_fall() {
            self.piece.y += 1;
        }
    }

    pub fn speed_up(&mut self) {
        self.speed -= 4.0;
        println!("NEW SPEED is {}", self.speed);
    }

    pub fn speed_down(&mut self) {
        self.speed += 4.0;
        println!("NEW SPEED is {}", self.speed);
    }

    pub fn grab_piece(&mut self) -> &'static tetrominos::Tetromino {
        if self.piece_bag.len() == 0 {
            self.piece_bag = new_tetromino_bag();
        }

        let mut rng = rand::thread_rng();
        let n1: usize = rng.gen_range(0..self.piece_bag.len());

        let p = self.piece_bag.swap_remove(n1);

        p
    }

    pub fn grab_next_piece(&mut self) -> Result<(), String> {
        self.piece.tetromino = self.next_piece;
        self.next_piece = self.grab_piece();
        self.piece.rotation = 0;
        self.piece.x = (self.play_field.well_x() + (self.play_field.cols / 2) - 2) as u16;
        self.piece.y = 2;
        self.piece.creep = 0.0;

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
                        playfield::Location::Filled(self.piece.tetromino.kind);
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

fn rand_tetromino() -> &'static tetrominos::Tetromino {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen_range(0..7);

    let t = match n1 {
        0 => tetrominos::from_kind(tetrominos::Kind::Stick),
        1 => tetrominos::from_kind(tetrominos::Kind::Square),
        2 => tetrominos::from_kind(tetrominos::Kind::Pyramid),
        3 => tetrominos::from_kind(tetrominos::Kind::Seven),
        4 => tetrominos::from_kind(tetrominos::Kind::Snake),
        5 => tetrominos::from_kind(tetrominos::Kind::Hook),
        6 => tetrominos::from_kind(tetrominos::Kind::Zig),
        _ => panic!("BAD ROBIT"),
    };

    t
}
