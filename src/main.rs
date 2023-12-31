use std::thread;
use std::time;

mod actions;
mod game;
mod playfield;
mod tetrominos;

extern crate sdl2;
use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::ttf;
use sdl2::video;

const SCREEN_WIDTH: u32 = 1800;
const SCREEN_HEIGHT: u32 = 1200;

const CELL_SIZE: i32 = 32;

fn tetromino_colour(kind: tetrominos::Kind) -> pixels::Color {
    match kind {
        tetrominos::Kind::Hook => pixels::Color::RGB(92, 101, 168),
        tetrominos::Kind::Pyramid => pixels::Color::RGB(161, 82, 153),
        tetrominos::Kind::Seven => pixels::Color::RGB(224, 127, 58),
        tetrominos::Kind::Snake => pixels::Color::RGB(100, 180, 82),
        tetrominos::Kind::Square => pixels::Color::RGB(241, 212, 72),
        tetrominos::Kind::Stick => pixels::Color::RGB(99, 196, 234),
        tetrominos::Kind::Zig => pixels::Color::RGB(220, 58, 53),
    }
}

impl playfield::Location {
    fn color(self) -> pixels::Color {
        match self {
            playfield::Location::Empty => pixels::Color::RGB(0, 0, 0),
            playfield::Location::Edge => pixels::Color::RGB(200, 200, 200),
            playfield::Location::Filled(k) => match k {
                tetrominos::Kind::Stick => pixels::Color::RGB(99, 196, 234),
                tetrominos::Kind::Square => pixels::Color::RGB(241, 212, 72),
                tetrominos::Kind::Pyramid => pixels::Color::RGB(161, 82, 153),
                tetrominos::Kind::Seven => pixels::Color::RGB(224, 127, 58),
                tetrominos::Kind::Snake => pixels::Color::RGB(100, 180, 82),
                tetrominos::Kind::Hook => pixels::Color::RGB(92, 101, 168),
                tetrominos::Kind::Zig => pixels::Color::RGB(220, 58, 53),
            },
        }
    }
}

fn draw_shape(
    canvas: &mut render::Canvas<video::Window>,
    s: playfield::Shape,
    colour: pixels::Color,
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
            let _ = canvas.fill_rect(rect::Rect::new(
                x + (col as i32 * size),
                y + (row as i32 * size),
                size as u32,
                size as u32,
            ));
        }
    }
}

fn draw_playfield(canvas: &mut render::Canvas<video::Window>, pf: &playfield::PlayField) {
    let size: i32 = CELL_SIZE;

    let start_x: i32 = (SCREEN_WIDTH as i32 - (CELL_SIZE * 10)) / 2;
    let start_y: i32 = 1;

    let width: u32 = (size * pf.matrix[0].len() as i32) as u32 + 2;
    let height: u32 = (size * pf.matrix.len() as i32) as u32 + 2;

    for row in 0..(pf.matrix.len()) {
        for col in 0..(pf.matrix[row].len()) {
            canvas.set_draw_color(pixels::Color::RGB(20, 20, 20));
            let _ = canvas.draw_rect(rect::Rect::new(
                start_x + (col as i32 * size),
                start_y + (row as i32 * size),
                size as u32,
                size as u32,
            ));

            if pf.matrix[row][col] == playfield::Location::Empty {
                continue;
            }
            canvas.set_draw_color((&pf.matrix[row][col]).color());
            let _ = canvas.fill_rect(rect::Rect::new(
                start_x + (col as i32 * size),
                start_y + (row as i32 * size),
                size as u32,
                size as u32,
            ));
        }
    }

    canvas.set_draw_color(pixels::Color::RGB(72, 72, 72));
    let _ = canvas.draw_rect(rect::Rect::new(start_x, start_y, width, height));
}

fn draw_game(canvas: &mut render::Canvas<video::Window>, game: &game::Game) {
    draw_playfield(canvas, &game.play_field);
}

fn render_fps(canvas: &mut render::Canvas<video::Window>, font: &ttf::Font, fps: f64, lc: usize) {
    let texture_creator = canvas.texture_creator();

    let surface = font
        .render(format!("{:.2} fps", fps).as_str())
        .blended(pixels::Color::RGBA(0, 255, 0, 255))
        .map_err(|e| e.to_string())
        .unwrap();
    let fps_tex = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let fps_target = rect::Rect::new(0, 0, 180, 80);

    let _ = canvas.copy(&fps_tex, None, Some(fps_target));

    let surface2 = font
        .render(format!("lines cleared: {}", lc).as_str())
        .blended(pixels::Color::RGBA(122, 255, 122, 255))
        .map_err(|e| e.to_string())
        .unwrap();
    let fps_tex2 = texture_creator
        .create_texture_from_surface(&surface2)
        .map_err(|e| e.to_string())
        .unwrap();

    let fps_target2 = rect::Rect::new(0, 80, 280, 80);

    let _ = canvas.copy(&fps_tex2, None, Some(fps_target2));
}

fn render_text(
    canvas: &mut render::Canvas<video::Window>,
    font: &ttf::Font,
    text: String,
    x: i32,
    y: i32,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font
        .render(&text)
        .blended(pixels::Color::RGBA(0, 255, 0, 255))
        .map_err(|e| e.to_string())
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let char_width = 16;
    let char_height = 16;
    let target = rect::Rect::new(x, y, (char_width * text.len()) as u32, char_height);

    let _ = canvas.copy(&texture, None, Some(target));
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

    let smaller_font =
        ttf_context.load_font("/usr/share/fonts/TTF/PressStart2P-Regular.ttf", 64)?;

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
    let dt: f64 = 1.0 / 240.0;

    let mut start_time = time::Instant::now();
    let mut accumulator: f64 = 0.0;

    let mut game = game::Game::new()?;

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
                    event::Event::Quit { .. } => break 'main,

                    event::Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        match keycode {
                            keyboard::Keycode::Escape => break 'main,
                            keyboard::Keycode::Space => game.paused = !game.paused,

                            // these are game actions and need to be handled in the game sim eventually
                            keyboard::Keycode::Kp7 => game.queue_action(actions::Action::MoveLeft),
                            keyboard::Keycode::Kp9 => game.queue_action(actions::Action::MoveRight),
                            keyboard::Keycode::Kp4 => game.queue_action(actions::Action::Drop),
                            keyboard::Keycode::Kp5 => game.queue_action(actions::Action::MoveDown),
                            keyboard::Keycode::Kp6 => {
                                let _ = game.grab_next_piece();
                            }
                            keyboard::Keycode::Kp8 => game.queue_action(actions::Action::Rotate),
                            keyboard::Keycode::KpPlus => game.speed_up(),
                            keyboard::Keycode::KpMinus => game.speed_down(),

                            _ => (),
                        }
                    }

                    event::Event::MouseButtonDown { .. } => {
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
            1300,
            160,
        );

        render_fps(&mut canvas, &font, frame_rate, game.score_lines_cleared);
        render_text(
            &mut canvas,
            &smaller_font,
            "Next Piece:".to_string(),
            1300,
            100,
        );

        canvas.present();
    }

    Ok(())
}
