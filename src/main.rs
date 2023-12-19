type Map = Vec<Vec<Location>>;

use std::cmp::min;
use std::thread;

mod types;
use types::*;

mod playfield;
use playfield::PlayField;

mod pieces;
use pieces::J_PIECE;

mod tetrominos;

use std::collections::hash_map::HashMap;
use std::collections::HashSet;

extern crate sdl2;

// use std::mem;
use std::time::Duration;
use std::time::Instant;

use rand::Rng;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

const SCREEN_WIDTH: u32 = 1800;
const SCREEN_HEIGHT: u32 = 1200;

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

const CELL_SIZE: i32 = 44;

type Arena = [[u8; 20]; 40];

#[derive(Debug, Copy, Clone, PartialEq)]
enum Location {
    Empty,
    Straight,
    Square,
    T,
    L,
    Skew,
    J,
    Z,
}

fn tetromino_colour(kind: tetrominos::Kind) -> pixels::Color {
    match kind {
        tetrominos::Kind::Hook => Color::RGB(92, 101, 168),
        tetrominos::Kind::Pyramid => Color::RGB(161, 82, 153),
        tetrominos::Kind::Seven => Color::RGB(224, 127, 58),
        tetrominos::Kind::Snake => Color::RGB(100, 180, 82),
        tetrominos::Kind::Square => Color::RGB(241, 212, 72),
        tetrominos::Kind::Stick => Color::RGB(99, 196, 234),
        tetrominos::Kind::Zig => Color::RGB(220, 58, 53),
    }
}

// Color::RGB(99, 196, 234) - straight
// Color::RGB(92, 101, 168) - j
// Color::RGB(224, 127, 58) - l
// Color::RGB(241, 212, 72) - square
// Color::RGB(100, 180, 82) - skew
// Color::RGB(161, 82, 153) - t/pyramid
// Color::RGB(220, 58, 53) - z
impl Location {
    fn color(self) -> Color {
        match self {
            Location::Empty => Color::RGB(0, 0, 0),
            Location::Straight => Color::RGB(99, 196, 234),
            Location::Square => Color::RGB(241, 212, 72),
            Location::T => Color::RGB(161, 82, 153),
            Location::L => Color::RGB(224, 127, 58),
            Location::Skew => Color::RGB(100, 180, 82),
            Location::J => Color::RGB(92, 101, 168),
            Location::Z => Color::RGB(220, 58, 53),
        }
    }
}

const NUM_PIECES: u8 = 7;

fn rand_piece() -> &'static Piece {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen_range(0..NUM_PIECES);

    let p = match n1 {
        0 => &STRAIGHT_PIECE,
        1 => &SQUARE_PIECE,
        2 => &L_PIECE,
        3 => &SKEW_PIECE,
        4 => &T_PIECE,
        5 => &J_PIECE,
        6 => &Z_PIECE,
        _ => panic!("rand_piece BAD"),
    };

    p
}

fn new_map(cols: usize, rows: usize) -> Map {
    vec![vec![Location::Empty; cols]; rows]
}

struct Game<'g> {
    speed: f64,
    paused: bool,
    map: Map,
    play_field: PlayField,
    next_piece: &'g Piece,
    piece_bag: Vec<&'static Piece>,
    piece: &'g Piece,
    piece_pos: Position,
    piece_creep: f64,
    piece_rotation: usize,

    score_lines_cleared: usize,
}

fn new_piece_bag() -> Vec<&'static Piece> {
    let piece_bag = vec![
        &STRAIGHT_PIECE,
        &SQUARE_PIECE,
        &L_PIECE,
        &SKEW_PIECE,
        &T_PIECE,
        &J_PIECE,
        &Z_PIECE,
    ];

    return piece_bag;
}

impl<'g> Game<'g> {
    fn new() -> Game<'g> {
        Game {
            speed: 30.0,
            paused: false,
            map: new_map(10, 24),
            play_field: PlayField::new(24, 10),

            piece: rand_piece(),
            next_piece: rand_piece(),
            piece_bag: new_piece_bag(),

            piece_pos: Position { x: 4, y: 0 },
            piece_creep: 0.0,
            piece_rotation: 0,

            score_lines_cleared: 0,
        }
    }

    fn tetromino(&self) -> &Tetromino {
        piece_tetro(self.piece, self.piece_rotation)
    }

    fn speed_up(&mut self) {
        self.speed -= 4.0;
        println!("NEW SPEED is {}", self.speed);
    }

    fn speed_down(&mut self) {
        self.speed += 4.0;
        println!("NEW SPEED is {}", self.speed);
    }

    fn grab_piece(&mut self) -> &'static Piece {
        if self.piece_bag.len() == 0 {
            self.piece_bag = new_piece_bag();
        }

        let mut rng = rand::thread_rng();
        let n1: usize = rng.gen_range(0..self.piece_bag.len());

        let p = self.piece_bag.swap_remove(n1);

        p
    }
}

fn draw_shape(canvas: &mut Canvas<Window>, s: Shape, colour: Color, size: i32, x: i32, y: i32) {
    canvas.set_draw_color(colour);
    for row in 0..4 {
        for col in 0..4 {
            if s[row][col] == 0 {
                continue;
            }
            canvas.fill_rect(Rect::new(
                x + (col as i32 * size),
                y + (row as i32 * size),
                size as u32,
                size as u32,
            ));
        }
    }
}

// Color::RGB(99, 196, 234) - straight
// Color::RGB(92, 101, 168) - j
// Color::RGB(224, 127, 58) - l
// Color::RGB(241, 212, 72) - square
// Color::RGB(100, 180, 82) - skew
// Color::RGB(161, 82, 153) - t/pyramid
// Color::RGB(220, 58, 53) - z

fn piece_colour(p: &Piece) -> Color {
    match p {
        Piece::Straight(_) => Color::RGB(99, 196, 234),
        Piece::Square(_) => Color::RGB(241, 212, 72),
        Piece::L(_) => Color::RGB(224, 127, 58),
        Piece::Skew(_) => Color::RGB(100, 180, 82),
        Piece::T(_) => Color::RGB(161, 82, 153),
        Piece::J(_) => Color::RGB(92, 101, 168),
        Piece::Z(_) => Color::RGB(220, 58, 53),
    }
}

fn piece_location(p: &Piece) -> Location {
    match p {
        Piece::Straight(_) => Location::Straight,
        Piece::Square(_) => Location::Square,
        Piece::L(_) => Location::L,
        Piece::Skew(_) => Location::Skew,
        Piece::T(_) => Location::T,
        Piece::J(_) => Location::J,
        Piece::Z(_) => Location::Z,
    }
}

fn piece_tetro(p: &Piece, rot: usize) -> &Tetromino {
    match p {
        Piece::Straight(sp) => &(sp.tetros[rot]),
        Piece::Square(sp) => &(sp.tetros[rot]),
        Piece::L(lp) => &(lp.tetros[rot]),
        Piece::Skew(sp) => &(sp.tetros[rot]),
        Piece::T(tp) => &(tp.tetros[rot]),
        Piece::J(jp) => &(jp.tetros[rot]),
        Piece::Z(zp) => &(zp.tetros[rot]),
        // _ => panic!("piece_tetro BAD - this is getting hairy"),
    }
}

fn piece_rotations(p: &Piece) -> usize {
    return p.rotations();
}

fn draw_piece(canvas: &mut Canvas<Window>, piece: &Piece, pos: &Position, rot: usize) {
    let size = CELL_SIZE as i32;
    let start_x: i32 = (SCREEN_WIDTH as i32 - (CELL_SIZE * 10)) / 2;
    let start_y: i32 = 1;

    draw_shape(
        canvas,
        piece_tetro(piece, rot).shape,
        piece_colour(piece),
        size,
        start_x + (pos.x * size),
        start_y + (pos.y * size),
    )
}

fn draw_map(canvas: &mut Canvas<Window>, map: &Map) {
    let size: i32 = CELL_SIZE;

    let start_x: i32 = (SCREEN_WIDTH as i32 - (CELL_SIZE * 10)) / 2;
    let start_y: i32 = 1;

    let width: u32 = (size * map[0].len() as i32) as u32 + 2;
    let height: u32 = (size * map.len() as i32) as u32 + 2;

    for row in 0..map.len() {
        for col in 0..map[row].len() {
            canvas.set_draw_color(Color::RGB(20, 20, 20));
            canvas.draw_rect(Rect::new(
                start_x + (col as i32 * size),
                start_y + (row as i32 * size),
                size as u32,
                size as u32,
            ));

            if map[row][col] == Location::Empty {
                continue;
            }
            canvas.set_draw_color((&map[row][col]).color());
            canvas.fill_rect(Rect::new(
                start_x + (col as i32 * size),
                start_y + (row as i32 * size),
                size as u32,
                size as u32,
            ));
        }
    }

    canvas.set_draw_color(Color::RGB(72, 72, 72));
    canvas.draw_rect(Rect::new(start_x, start_y, width, height));
}

fn draw_game(canvas: &mut Canvas<Window>, game: &Game) {
    draw_map(canvas, &game.map);
}

fn imprint_piece(game: &mut Game) {
    let r_offset = game.piece_pos.y as usize;
    let c_offset = game.piece_pos.x as usize;

    for r in 0..4 {
        for c in 0..4 {
            if piece_tetro(game.piece, game.piece_rotation).shape[r][c] == 1 {
                game.map[r + r_offset][c + c_offset] = piece_location(&game.piece);
            }
        }
    }
}

fn clear_full_rows(game: &mut Game) {
    for r in 0..game.map.len() {
        let mut col_count = 0;
        for c in 0..game.map[r].len() {
            if game.map[r][c] != Location::Empty {
                col_count += 1;
            }
        }
        if col_count == game.map[r].len() {
            for x in 0..game.map[r].len() {
                game.map[r][x] = Location::Empty;
            }
            game.score_lines_cleared += 1;
        }
    }
}

fn bottom_row(shape: Shape) -> (usize, [u8; 4]) {
    for r in (0..4).rev() {
        for c in 0..4 {
            if shape[r][c] != 0 {
                return (r, shape[r]);
            }
        }
    }

    println!("{:?} BULL", shape);

    panic!("BAD CODER")
}

fn drop_one(game: &mut Game) {
    if can_fall(game) {
        game.piece_pos.y += 1;
    }
}

fn drop_fast(game: &mut Game) {
    while can_fall(game) {
        game.piece_pos.y += 1;
    }
}

fn can_fall(game: &mut Game) -> bool {
    return !has_collission(
        &game.map,
        game.piece_pos.y as u32 + 1,
        game.piece_pos.x as u32,
        piece_tetro(game.piece, game.piece_rotation),
    );
}

fn collapse(game: &mut Game) {
    for r in (0..game.map.len()).rev() {
        let mut has_block = false;
        for c in 0..game.map[r].len() {
            if game.map[r][c] != Location::Empty {
                has_block = true;
                break;
            }
        }

        if !has_block {
            for ir in (1..=r).rev() {
                for c in 0..game.map[0].len() {
                    game.map[ir][c] = game.map[ir - 1][c];
                }
            }
        }
    }
}

fn next_piece(game: &mut Game) {
    game.piece = game.next_piece;
    game.next_piece = game.grab_piece();
    game.piece_rotation = 0;
    game.piece_pos.x = 4;
    game.piece_pos.y = 0;
    game.piece_creep = 0.0;
}

fn rotate(game: &mut Game) {
    let next_rotation = game.piece.rotation_after(game.piece_rotation);
    let next_tetro = game.piece.tetro(next_rotation);

    if !has_collission(
        &game.map,
        game.piece_pos.y as u32,
        game.piece_pos.x as u32,
        next_tetro,
    ) {
        game.piece_rotation = next_rotation;
    }
}

fn game_sim(game: &mut Game, t: f64, dt: f64, acc: f64) {
    // println!("SIMULATING GAME ENGINE... {:?} {:?} {:?}", t, dt, acc);

    game.piece_creep += dt;
    if game.piece_creep > dt * game.speed {
        // move the piece
        game.piece_creep = 0.0;

        let t = piece_tetro(game.piece, game.piece_rotation);
        let bottom = game.piece_pos.y + t.height as i32;
        if bottom as usize == game.map.len() {
            // we are already on the floor so leave us and create a new piece

            // overlay the shape + position onto the map
            imprint_piece(game);

            next_piece(game);
        } else {
            if can_fall(game) {
                game.piece_pos.y += 1;
            } else {
                imprint_piece(game);
                next_piece(game);
            }
        }
        clear_full_rows(game);
        collapse(game);
    }

    // if game.piece_pos.y < 30 {}
}

fn collission_matrix(map: &Map, y: u32, x: u32, tetro: &Tetromino) -> bool {
    let width = tetro.width as u32;
    let height = tetro.height as u32;

    if y + height > map.len() as u32 {
        return true;
    }

    if x + width > map[0].len() as u32 {
        return true;
    }

    for r in 0..height {
        for c in 0..width {
            if tetro.shape[r as usize][c as usize] == 0 {
                continue;
            }
            if map[(y + r) as usize][(x + c) as usize] != Location::Empty {
                return true;
            }
        }
    }

    false
}

fn has_collission(map: &Map, y: u32, x: u32, tetro: &Tetromino) -> bool {
    return collission_matrix(map, y, x, tetro);
}

fn left(game: &mut Game) {
    if game.piece_pos.x > 0 {
        game.piece_pos.x -= 1;
    }
}

fn right(game: &mut Game) {
    if game.piece_pos.x < game.map[0].len() as i32 - (game.tetromino().width as i32) {
        game.piece_pos.x += 1;
    }
}

fn down(game: &mut Game) {
    if can_fall(game) {
        game.piece_pos.y += 1;
    }
    let bottom = game.piece_pos.y + game.tetromino().height as i32;
    if bottom < game.map.len() as i32 {}
}

fn render_fps(canvas: &mut Canvas<Window>, font: &Font, fps: f64, lc: usize) {
    let texture_creator = canvas.texture_creator();

    let surface = font
        .render(format!("{:.2} fps", fps).as_str())
        .blended(Color::RGBA(0, 255, 0, 255))
        .map_err(|e| e.to_string())
        .unwrap();
    let fps_tex = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let fps_target = Rect::new(0, 0, 180, 80);

    canvas.copy(&fps_tex, None, Some(fps_target));

    let surface2 = font
        .render(format!("lines cleared: {}", lc).as_str())
        .blended(Color::RGBA(122, 255, 122, 255))
        .map_err(|e| e.to_string())
        .unwrap();
    let fps_tex2 = texture_creator
        .create_texture_from_surface(&surface2)
        .map_err(|e| e.to_string())
        .unwrap();

    let fps_target2 = Rect::new(0, 80, 280, 80);

    canvas.copy(&fps_tex2, None, Some(fps_target2));
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut font = ttf_context.load_font(
        "/usr/share/fonts/adobe-source-code-pro/SourceCodePro-Regular.otf",
        128,
    )?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    println!(
        "video driver = {:?}, display name = {:?}",
        video_subsys.current_video_driver(),
        video_subsys.display_name(0)
    );
    let window = video_subsys
        .window("Panda Tetris", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));

    canvas.present();

    let mut lastx = 0;
    let mut lasty = 0;

    canvas.present();

    let mut events = sdl_context.event_pump()?;

    // println!("{:?} {:?}", L, T);

    let mut total = 0;
    let mut frames = 0;

    let mut t: f64 = 0.0;
    let mut dt: f64 = 1.0 / 120.0;

    let mut startTime = Instant::now();
    let mut accumulator: f64 = 0.0;

    let mut game = Game::new();

    'main: loop {
        frames = frames + 1;
        // let start = Instant::now();

        let now = Instant::now();
        let mut frame_time = now - startTime;
        let frame_rate = 1000000.0 / frame_time.as_micros() as f64;
        // println!("frames = {:?}, frame time = {:?}, frame rate = {:?}", frames, frame_time, frame_rate);
        if frame_time.as_secs_f64() > 0.25 {
            println!("******************************************************* SLOW");
            frame_time = Duration::from_millis(250);
            // 0.25;
        }
        startTime = now;
        accumulator = accumulator + frame_time.as_secs_f64();

        let mut acc_runs = 0;
        while accumulator >= dt {
            acc_runs += 1;
            // simulation things
            t += dt;
            accumulator -= dt;

            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        match keycode {
                            Keycode::Escape => break 'main,
                            Keycode::Space => game.paused = !game.paused,

                            // these are game actions and need to be handled in the game sim eventually
                            Keycode::Kp7 => left(&mut game),
                            Keycode::Kp9 => right(&mut game), // left
                            Keycode::Kp4 => drop_fast(&mut game),
                            Keycode::Kp5 => drop_one(&mut game),
                            Keycode::Kp6 => next_piece(&mut game),
                            Keycode::Kp8 => rotate(&mut game),
                            Keycode::KpPlus => game.speed_up(),
                            Keycode::KpMinus => game.speed_down(),

                            _ => (),
                        }
                    }

                    Event::MouseButtonDown { x, y, .. } => {
                        next_piece(&mut game);
                    }

                    _ => {}
                }
            }

            if !game.paused {
                game_sim(&mut game, t, dt, accumulator);
            }
        }

        acc_runs = 0;

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        draw_game(&mut canvas, &game);

        thread::sleep_ms(1);

        draw_piece(
            &mut canvas,
            &game.piece,
            &game.piece_pos,
            game.piece_rotation,
        );

        draw_shape(
            &mut canvas,
            piece_tetro(&game.next_piece, 0).shape,
            piece_colour(&game.next_piece),
            CELL_SIZE,
            1200,
            100,
        );

        render_fps(&mut canvas, &font, frame_rate, game.score_lines_cleared);

        canvas.present();
    }

    Ok(())
}
