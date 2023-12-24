use crate::playfield::PlayField;
use crate::tetrominos;

use rand::Rng;

#[derive(Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Game<'g> {
    pub speed: f64,
    pub paused: bool,
    pub play_field: PlayField,
    pub next_piece: &'g tetrominos::Tetromino,
    pub piece_bag: Vec<&'static tetrominos::Tetromino>,
    pub piece: &'g tetrominos::Tetromino,
    pub piece_pos: Position,
    pub piece_creep: f64,
    pub piece_rotation: usize,

    pub score_lines_cleared: usize,
}

impl<'g> Game<'g> {
    pub fn new() -> Game<'g> {
        Game {
            speed: 30.0,
            paused: false,
            play_field: PlayField::new(24, 10),

            piece: rand_tetromino(),
            next_piece: rand_tetromino(),
            piece_bag: new_tetromino_bag(),

            piece_pos: Position { x: 4, y: 0 },
            piece_creep: 0.0,
            piece_rotation: 0,

            score_lines_cleared: 0,
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
