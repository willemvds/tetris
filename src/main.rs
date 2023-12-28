use std::thread;

use tetrominos::Kind;

mod playfield;
//use playfield::Location;
//use playfield::PlayField;
//use playfield::Shape;

mod tetrominos;

mod game;
//use game::Game;

extern crate sdl2;

// use std::mem;
use std::time;
//use std::time::Duration;
//use std::time::Instant;

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

impl playfield::Location {
    fn color(self) -> Color {
        match self {
            playfield::Location::Empty => Color::RGB(0, 0, 0),
            playfield::Location::Edge => Color::RGB(200, 200, 200),
            playfield::Location::Filled(k) => match k {
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

fn draw_shape(
    canvas: &mut Canvas<Window>,
    s: playfield::Shape,
    colour: Color,
    size: i32,
    x: i32,
    y: i32,
) {
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

fn draw_playfield(canvas: &mut Canvas<Window>, pf: &playfield::PlayField) {
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

            if pf.matrix[row][col] == playfield::Location::Empty {
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

fn draw_game(canvas: &mut Canvas<Window>, game: &game::Game) {
    draw_playfield(canvas, &game.play_field);
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

    let mut start_time = time::Instant::now();
    let mut accumulator: f64 = 0.0;

    let mut game = game::Game::new();

    'main: loop {
        frames = frames + 1;
        // let start = Instant::now();

        let now = time::Instant::now();
        let mut frame_time = now - start_time;
        let frame_rate = 1000000.0 / frame_time.as_micros() as f64;
        // println!("frames = {:?}, frame time = {:?}, frame rate = {:?}", frames, frame_time, frame_rate);
        if frame_time.as_secs_f64() > 0.25 {
            println!("******************************************************* SLOW");
            frame_time = time::Duration::from_millis(250);
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
                            Keycode::Kp7 => game.move_left(),
                            Keycode::Kp9 => game.move_right(), // left
                            Keycode::Kp4 => game.drop_fast(),
                            Keycode::Kp5 => game.drop_one(),
                            Keycode::Kp6 => game.grab_next_piece(),
                            Keycode::Kp8 => game.rotate(),
                            Keycode::KpPlus => game.speed_up(),
                            Keycode::KpMinus => game.speed_down(),

                            _ => (),
                        }
                    }

                    Event::MouseButtonDown { .. } => {
                        //                        next_piece(&mut game);
                    }

                    _ => {}
                }
            }

            if !game.paused {
                game.sim(t, dt, accumulator);
            }
        }

        if acc_runs > 1 {
            println!("Multiple({acc_runs}) simulations during single frame.");
        }

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        draw_game(&mut canvas, &game);

        thread::sleep(time::Duration::from_millis(1));

        //    let size = CELL_SIZE as i32;
        let start_x: i32 = (SCREEN_WIDTH as i32 - (CELL_SIZE * 10)) / 2;
        let start_y: i32 = 1;

        draw_shape(
            &mut canvas,
            *game.piece.form(),
            tetromino_colour(game.piece.tetromino.kind),
            CELL_SIZE,
            start_x + (game.piece.x as i32 * CELL_SIZE),
            start_y + (game.piece.y as i32 * CELL_SIZE),
        );

        draw_shape(
            &mut canvas,
            game.next_piece.forms[0],
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
