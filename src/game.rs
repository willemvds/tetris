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

pub struct Game {
    pub speed: f64,
    pub paused: bool,
    pub play_field: playfield::PlayField,
    pub next_piece: &'static tetrominos::Tetromino,
    pub piece_bag: Vec<&'static tetrominos::Tetromino>,
    pub piece: Piece,
    pub score_lines_cleared: usize,
}

impl Game {
    pub fn new() -> Game {
        Game {
            speed: 30.0,
            paused: false,
            play_field: playfield::PlayField::new(24, 10),

            piece: Piece::new(rand_tetromino()),
            next_piece: rand_tetromino(),
            piece_bag: new_tetromino_bag(),

            score_lines_cleared: 0,
        }
    }

    pub fn sim(&mut self, _t: f64, dt: f64, _acc: f64) {
        // println!("SIMULATING GAME ENGINE... {:?} {:?} {:?}", t, dt, acc);

        self.piece.creep += dt;
        if self.piece.creep > dt * self.speed {
            // move the piece
            self.piece.creep = 0.0;

            let bottom = self.piece.y + 4;
            if bottom as usize == self.play_field.matrix.len() {
                // we are already on the floor so leave us and create a new piece

                // overlay the shape + position onto the map
                self.imprint_piece();

                self.grab_next_piece();
            } else {
                if self.can_fall() {
                    self.piece.y += 1;
                } else {
                    self.imprint_piece();
                    self.grab_next_piece();
                }
            }
            self.clear_full_rows();
            self.collapse();
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

    fn clear_full_rows(&mut self) {
        let row_offset = self.play_field.well_y();
        let col_offset = self.play_field.well_x();

        for row in row_offset..self.play_field.rows + row_offset {
            let mut col_count = 0;
            for col in col_offset..self.play_field.cols + col_offset {
                if self.play_field.matrix[row][col] != playfield::Location::Empty {
                    col_count += 1;
                }
            }
            if col_count == self.play_field.cols {
                for clear_col in col_offset..self.play_field.cols + col_offset {
                    self.play_field.matrix[row][clear_col] = playfield::Location::Empty;
                }
                self.score_lines_cleared += 1;
            }
        }
    }

    fn collapse(&mut self) {
        let row_offset = self.play_field.well_y();
        let col_offset = self.play_field.well_x();

        for row in (row_offset..self.play_field.rows + row_offset).rev() {
            let mut has_block = false;
            for col in col_offset..self.play_field.cols + col_offset {
                if self.play_field.matrix[row][col] != playfield::Location::Empty {
                    has_block = true;
                    break;
                }
            }

            if !has_block {
                for ir in (1..=row).rev() {
                    for c in col_offset..self.play_field.cols + col_offset {
                        self.play_field.matrix[ir][c] = self.play_field.matrix[ir - 1][c];
                    }
                }
            }
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

    pub fn grab_next_piece(&mut self) {
        self.piece.tetromino = self.next_piece;
        self.next_piece = self.grab_piece();
        self.piece.rotation = 0;
        self.piece.x = 4;
        self.piece.y = 0;
        self.piece.creep = 0.0;
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
