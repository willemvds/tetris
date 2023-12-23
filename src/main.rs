use std::thread;

use tetrominos::Kind;

mod playfield;
use playfield::Location;
use playfield::PlayField;
use playfield::Shape;

use tetrominos::Tetromino;

mod tetrominos;

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

impl Location {
    fn color(self) -> Color {
        match self {
            Location::Empty => Color::RGB(0, 0, 0),
            Location::Edge => Color::RGB(200, 200, 200),
            Location::Filled(k) => match k {
                Kind::Stick => Color::RGB(99, 196, 234),
                Kind::Square => Color::RGB(241, 212, 72),
                Kind::Pyramid => Color::RGB(161, 82, 153),
                Kind::Seven => Color::RGB(224, 127, 58),
                Kind::Snake => Color::RGB(100, 180, 82),
                Kind::Hook => Color::RGB(92, 101, 168),
                Kind::Zig => Color::RGB(220, 58, 53),
            },
        }
    }
}

const NUM_PIECES: u8 = 7;

fn rand_tetromino() -> &'static tetrominos::Tetromino {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen_range(0..NUM_PIECES);

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

struct Game<'g> {
    speed: f64,
    paused: bool,
    play_field: PlayField,
    next_piece: &'g Tetromino,
    piece_bag: Vec<&'static Tetromino>,
    piece: &'g Tetromino,
    piece_pos: Position,
    piece_creep: f64,
    piece_rotation: usize,

    score_lines_cleared: usize,
}

fn new_tetromino_bag() -> Vec<&'static Tetromino> {
    return vec![
        tetrominos::from_kind(Kind::Stick),
        tetrominos::from_kind(Kind::Square),
        tetrominos::from_kind(Kind::Pyramid),
        tetrominos::from_kind(Kind::Seven),
        tetrominos::from_kind(Kind::Snake),
        tetrominos::from_kind(Kind::Hook),
        tetrominos::from_kind(Kind::Zig),
    ];
}

impl<'g> Game<'g> {
    fn new() -> Game<'g> {
        Game {
            speed: 30.0,
            paused: false,
            //            map: new_map(10, 24),
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

    fn speed_up(&mut self) {
        self.speed -= 4.0;
        println!("NEW SPEED is {}", self.speed);
    }

    fn speed_down(&mut self) {
        self.speed += 4.0;
        println!("NEW SPEED is {}", self.speed);
    }

    fn grab_piece(&mut self) -> &'static Tetromino {
        if self.piece_bag.len() == 0 {
            self.piece_bag = new_tetromino_bag();
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
            let _ = canvas.fill_rect(Rect::new(
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

fn piece_location(k: Kind) -> Location {
    match k {
        Kind::Stick => Location::Filled(tetrominos::Kind::Stick),
        Kind::Square => Location::Filled(tetrominos::Kind::Square),
        Kind::Seven => Location::Filled(tetrominos::Kind::Seven),
        Kind::Snake => Location::Filled(tetrominos::Kind::Snake),
        Kind::Pyramid => Location::Filled(tetrominos::Kind::Pyramid),
        Kind::Hook => Location::Filled(tetrominos::Kind::Hook),
        Kind::Zig => Location::Filled(tetrominos::Kind::Zig),
    }
}

fn piece_shape(k: Kind, rot: usize) -> &'static Shape {
    let t = tetrominos::from_kind(k);

    return &t.forms[rot];
}

//fn piece_tetro(p: &Piece, rot: usize) -> &Tetromino {
//    match p {
//        Piece::Straight(sp) => &(sp.tetros[rot]),
//        Piece::Square(sp) => &(sp.tetros[rot]),
//        Piece::L(lp) => &(lp.tetros[rot]),
//        Piece::Skew(sp) => &(sp.tetros[rot]),
//        Piece::T(tp) => &(tp.tetros[rot]),
//        Piece::J(jp) => &(jp.tetros[rot]),
//        Piece::Z(zp) => &(zp.tetros[rot]),
//    }
//}

fn draw_piece(canvas: &mut Canvas<Window>, t: &Tetromino, pos: &Position, rot: usize) {
    let size = CELL_SIZE as i32;
    let start_x: i32 = (SCREEN_WIDTH as i32 - (CELL_SIZE * 10)) / 2;
    let start_y: i32 = 1;

    draw_shape(
        canvas,
        *piece_shape(t.kind, rot),
        tetromino_colour(t.kind),
        size,
        start_x + (pos.x * size),
        start_y + (pos.y * size),
    )
}

fn draw_playfield(canvas: &mut Canvas<Window>, pf: &PlayField) {
    let size: i32 = CELL_SIZE;

    let start_x: i32 = (SCREEN_WIDTH as i32 - (CELL_SIZE * 10)) / 2;
    let start_y: i32 = 1;

    let width: u32 = (size * pf.matrix[0].len() as i32) as u32 + 2;
    let height: u32 = (size * pf.matrix.len() as i32) as u32 + 2;

    for row in 0..(pf.matrix.len()) {
        for col in 0..(pf.matrix[row].len()) {
            canvas.set_draw_color(Color::RGB(20, 20, 20));
            let _ = canvas.draw_rect(Rect::new(
                start_x + (col as i32 * size),
                start_y + (row as i32 * size),
                size as u32,
                size as u32,
            ));

            if pf.matrix[row][col] == Location::Empty {
                continue;
            }
            canvas.set_draw_color((&pf.matrix[row][col]).color());
            let _ = canvas.fill_rect(Rect::new(
                start_x + (col as i32 * size),
                start_y + (row as i32 * size),
                size as u32,
                size as u32,
            ));
        }
    }

    canvas.set_draw_color(Color::RGB(72, 72, 72));
    let _ = canvas.draw_rect(Rect::new(start_x, start_y, width, height));
}

fn draw_game(canvas: &mut Canvas<Window>, game: &Game) {
    draw_playfield(canvas, &game.play_field);
}

fn imprint_piece(game: &mut Game) {
    let r_offset = game.piece_pos.y as usize;
    let c_offset = game.piece_pos.x as usize;

    for r in 0..4 {
        for c in 0..4 {
            if piece_shape(game.piece.kind, game.piece_rotation)[r][c] == 1 {
                game.play_field.matrix[r + r_offset][c + c_offset] =
                    piece_location(game.piece.kind);
            }
        }
    }
}

fn clear_full_rows(game: &mut Game) {
    for r in 0..game.play_field.matrix.len() {
        let mut col_count = 0;
        for c in 0..game.play_field.matrix[r].len() {
            if game.play_field.matrix[r][c] != Location::Empty {
                col_count += 1;
            }
        }
        if col_count == game.play_field.matrix[r].len() {
            for x in 0..game.play_field.matrix[r].len() {
                game.play_field.matrix[r][x] = Location::Empty;
            }
            game.score_lines_cleared += 1;
        }
    }
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
        &game.play_field,
        (game.piece_pos.y + 1).try_into().unwrap(),
        (game.piece_pos.x).try_into().unwrap(),
        piece_shape(game.piece.kind, game.piece_rotation),
    );
}

fn collapse(game: &mut Game) {
    for r in (0..game.play_field.matrix.len()).rev() {
        let mut has_block = false;
        for c in 0..game.play_field.matrix[r].len() {
            if game.play_field.matrix[r][c] != Location::Empty {
                has_block = true;
                break;
            }
        }

        if !has_block {
            for ir in (1..=r).rev() {
                for c in 0..game.play_field.matrix[0].len() {
                    game.play_field.matrix[ir][c] = game.play_field.matrix[ir - 1][c];
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
    let mut next_rotation = game.piece_rotation + 1;
    if next_rotation >= 4 {
        next_rotation = 0;
    }
    //    let next_tetro = game.piece.tetro(next_rotation);
    let next_shape = piece_shape(game.piece.kind, next_rotation);

    if !has_collission(
        &game.play_field,
        game.piece_pos.y.try_into().unwrap(),
        game.piece_pos.x.try_into().unwrap(),
        next_shape,
    ) {
        game.piece_rotation = next_rotation;
    }
}

fn game_sim(game: &mut Game, _t: f64, dt: f64, _acc: f64) {
    // println!("SIMULATING GAME ENGINE... {:?} {:?} {:?}", t, dt, acc);

    game.piece_creep += dt;
    if game.piece_creep > dt * game.speed {
        // move the piece
        game.piece_creep = 0.0;

        //        let t = piece_tetro(game.piece, game.piece_rotation);
        let bottom = game.piece_pos.y + 4;
        if bottom as usize == game.play_field.matrix.len() {
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

fn has_collission(pf: &PlayField, y: usize, x: usize, shape: &playfield::Shape) -> bool {
    return pf.collission_matrix(x, y, shape);
    // return collission_matrix(map, y, x, tetro);
}

fn left(game: &mut Game) {
    //    if game.piece_pos.x > 1 {
    game.piece_pos.x -= 1;
    //    }
}

fn right(game: &mut Game) {
    //    if game.piece_pos.x < game.play_field.matrix[0].len() as i32 - (game.tetromino().width as i32) {
    game.piece_pos.x += 1;
    //    }
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

    let _ = canvas.copy(&fps_tex, None, Some(fps_target));

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

    let _ = canvas.copy(&fps_tex2, None, Some(fps_target2));
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

    let _lastx = 0;
    let _lasty = 0;

    canvas.present();

    let mut events = sdl_context.event_pump()?;

    // println!("{:?} {:?}", L, T);

    let _total = 0;
    let mut frames = 0;

    let mut t: f64 = 0.0;
    let dt: f64 = 1.0 / 120.0;

    let mut start_time = Instant::now();
    let mut accumulator: f64 = 0.0;

    let mut game = Game::new();

    'main: loop {
        frames = frames + 1;
        // let start = Instant::now();

        let now = Instant::now();
        let mut frame_time = now - start_time;
        let frame_rate = 1000000.0 / frame_time.as_micros() as f64;
        // println!("frames = {:?}, frame time = {:?}, frame rate = {:?}", frames, frame_time, frame_rate);
        if frame_time.as_secs_f64() > 0.25 {
            println!("******************************************************* SLOW");
            frame_time = Duration::from_millis(250);
            // 0.25;
        }
        start_time = now;
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

                    Event::MouseButtonDown { .. } => {
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

        thread::sleep(Duration::from_millis(1));

        draw_piece(
            &mut canvas,
            &game.piece,
            &game.piece_pos,
            game.piece_rotation,
        );

        draw_shape(
            &mut canvas,
            *piece_shape(game.next_piece.kind, 0),
            tetromino_colour(game.next_piece.kind),
            CELL_SIZE,
            1200,
            100,
        );

        render_fps(&mut canvas, &font, frame_rate, game.score_lines_cleared);

        canvas.present();
    }

    Ok(())
}
