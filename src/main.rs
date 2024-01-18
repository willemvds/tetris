use std::env;
use std::fs;
use std::io;
use std::time;

mod actions;
mod game;
mod playfield;
mod preferences;
mod tetrominos;

extern crate sdl2;
use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::ttf;
use sdl2::video;

use serde::{Deserialize, Serialize};
use typetag;

#[derive(PartialEq)]
enum UIAction {
    Quit,
    HideConsole,
}

struct Console {
    history: Vec<String>,
    buffer: String,
}

impl Console {
    fn new() -> Console {
        Console {
            history: vec![],
            buffer: "".to_string(),
        }
    }

    fn process_events(&mut self, event_pump: &mut sdl2::EventPump) -> Vec<UIAction> {
        let mut ui_actions = vec![];
        for event in event_pump.poll_iter() {
            match event {
                event::Event::Quit { .. } => ui_actions.push(UIAction::Quit),
                event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    keyboard::Keycode::Escape => ui_actions.push(UIAction::Quit),
                    keyboard::Keycode::Backquote => ui_actions.push(UIAction::HideConsole),
                    _ => println!("Console got this keycode: {0}", keycode),
                },
                _ => (),
            }
        }

        ui_actions
    }

    fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        let (canvas_width, _) = canvas.window().size();
        canvas.set_draw_color(pixels::Color::RGBA(200, 200, 200, 200));
        let _ = canvas.fill_rect(rect::Rect::new(0, 0, canvas_width, 500));
    }
}

const UI_LAYER_GAME: u8 = 0b0001;
const UI_LAYER_CONSOLE: u8 = 0b0010;
const UI_LAYER_MENU: u8 = 0b0100;

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

fn draw_shape_triangles(
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
            let start_x = x + (col as i32 * size);
            let start_y = y + (row as i32 * size);
            let end_x = start_x + size;
            let end_y = start_y + size;
            let _ = canvas.draw_line(
                rect::Point::new(start_x, start_y),
                rect::Point::new(end_x, end_y),
            );
            let _ = canvas.draw_rect(rect::Rect::new(
                x + (col as i32 * size),
                y + (row as i32 * size),
                size as u32,
                size as u32,
            ));
        }
    }
}

fn draw_shape_outline(
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

            let start_x = x + (col as i32 * size);
            let start_y = y + (row as i32 * size);

            // draw_top_line
            if row == 0 || s[row - 1][col] == 0 {
                let _ = canvas.draw_line(
                    rect::Point::new(start_x, start_y),
                    rect::Point::new(start_x + size, start_y),
                );
            }

            // draw_right_line
            if col == 3 || s[row][col + 1] == 0 {
                let _ = canvas.draw_line(
                    rect::Point::new(start_x + size, start_y),
                    rect::Point::new(start_x + size, start_y + size),
                );
            }

            // draw_bottom_line
            if row == 3 || s[row + 1][col] == 0 {
                let _ = canvas.draw_line(
                    rect::Point::new(start_x, start_y + size),
                    rect::Point::new(start_x + size, start_y + size),
                );
            }

            // draw_left_line
            if col == 0 || s[row][col - 1] == 0 {
                let _ = canvas.draw_line(
                    rect::Point::new(start_x, start_y),
                    rect::Point::new(start_x, start_y + size),
                );
            }
        }
    }
}

fn draw_partial_shape(
    canvas: &mut render::Canvas<video::Window>,
    s: playfield::Shape,
    s_first_row: i16,
    colour: pixels::Color,
    size: i32,
    x: i32,
    y: i32,
) {
    canvas.set_draw_color(colour);
    for row in s_first_row as usize..4 {
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

fn draw_playfield(
    canvas: &mut render::Canvas<video::Window>,
    pf: &playfield::PlayField,
    size: i32,
) {
    let (canvas_width, _) = canvas.window().size();

    // The 3 is the left padding of the playfield.
    let start_x: i32 = (canvas_width as i32 / 2) - (size * pf.cols as i32 / 2) - (3 * size);
    let start_y: i32 = 1;

    let well_rows_start = pf.well_y();
    let well_rows_end = well_rows_start + pf.rows;
    let well_cols_start = pf.well_x();
    let well_cols_end = well_cols_start + pf.cols;

    let width: u32 = (size * pf.cols as i32) as u32 + 2;
    let height: u32 = (size * pf.rows as i32) as u32 + 2;

    for row in well_rows_start..well_rows_end {
        for col in well_cols_start..well_cols_end {
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
    let _ = canvas.draw_rect(rect::Rect::new(
        start_x + (well_cols_start as i32 * size),
        start_y + (well_rows_start as i32 * size),
        width,
        height,
    ));
}

fn draw_game(canvas: &mut render::Canvas<video::Window>, game: &game::Game, size: i32) {
    draw_playfield(canvas, &game.play_field, size);
}

fn render_text(
    canvas: &mut render::Canvas<video::Window>,
    font: &ttf::Font,
    colour: pixels::Color,
    x: i32,
    y: i32,
    text: String,
) {
    let texture_creator = canvas.texture_creator();

    let (char_width, char_height) = font.size_of_char('C').unwrap();

    let surface = font
        .render(&text)
        .blended(colour)
        .map_err(|e| e.to_string())
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let target = rect::Rect::new(x, y, char_width * text.len() as u32, char_height);

    let _ = canvas.copy(&texture, None, Some(target));
}

fn render_text_centered(
    canvas: &mut render::Canvas<video::Window>,
    font: &ttf::Font,
    colour: pixels::Color,
    x: i32,
    y: i32,
    text: String,
) {
    let texture_creator = canvas.texture_creator();

    let (char_width, char_height) = font.size_of_char('C').unwrap();

    let surface = font
        .render(&text)
        .blended(colour)
        .map_err(|e| e.to_string())
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let text_width = char_width * text.len() as u32;
    let target_x = x - (text_width / 2) as i32;
    let target_y = y - (char_height / 2) as i32;

    let target = rect::Rect::new(target_x, target_y, text_width, char_height);

    let _ = canvas.copy(&texture, None, Some(target));
}

struct Replay {
    recording: game::recordings::Recording,
}

#[derive(Serialize, Deserialize)]
struct ReplayPieces {
    pieces: Vec<tetrominos::Kind>,
    idx: usize,
}

impl ReplayPieces {
    fn new(replay: &Replay) -> ReplayPieces {
        let piece_events = replay
            .recording
            .events
            .iter()
            .filter(|ev| matches!(ev.kind, game::recordings::EventKind::PieceSpawned(_)));
        let pieces = piece_events
            .map(|ev| {
                if let game::recordings::EventKind::PieceSpawned(k) = ev.kind {
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

fn render_game(
    canvas: &mut render::Canvas<video::Window>,
    game: &mut game::Game,
    prefs: &preferences::Preferences,
    font: &ttf::Font,
) {
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    let (window_width, window_height) = canvas.window().size();
    let cell_size: i32 = (window_height / 30) as i32;

    draw_game(canvas, &game, cell_size);

    let start_x: i32 =
        (window_width as i32 / 2) - (cell_size * game.play_field.cols as i32 / 2) - (3 * cell_size);
    let start_y: i32 = 1;

    if game.piece.y < 4 {
        draw_partial_shape(
            canvas,
            *game.piece.form(),
            4 - game.piece.y as i16,
            tetromino_colour(game.piece.tetromino),
            cell_size,
            start_x + (game.piece.x as i32 * cell_size),
            start_y + (game.piece.y as i32 * cell_size),
        );
    } else {
        draw_shape(
            canvas,
            *game.piece.form(),
            tetromino_colour(game.piece.tetromino),
            cell_size,
            start_x + (game.piece.x as i32 * cell_size),
            start_y + (game.piece.y as i32 * cell_size),
        );
    }

    if game.drop_distance() > 0 {
        if prefs.drop_indicator == preferences::DropIndicatorStyle::Outline {
            draw_shape_outline(
                canvas,
                *game.piece.form(),
                tetromino_colour(game.piece.tetromino),
                cell_size,
                start_x + (game.piece.x as i32 * cell_size),
                start_y + (game.piece.y + game.drop_distance() as u16 - 1) as i32 * cell_size,
            )
        } else if prefs.drop_indicator == preferences::DropIndicatorStyle::Triangles {
            draw_shape_triangles(
                canvas,
                *game.piece.form(),
                tetromino_colour(game.piece.tetromino),
                cell_size,
                start_x + (game.piece.x as i32 * cell_size),
                start_y + (game.piece.y + game.drop_distance() as u16 - 1) as i32 * cell_size,
            )
        }
    }

    draw_shape(
        canvas,
        tetrominos::from_kind(game.next_piece).forms[0],
        tetromino_colour(game.next_piece),
        cell_size,
        start_x + (game.play_field.cols as i32 * cell_size) + (window_width as i32 / 10),
        start_y + (window_width as i32 / 10),
    );

    let bright_green = pixels::Color::RGBA(0, 255, 0, 255);
    let bright_red = pixels::Color::RGBA(255, 0, 0, 255);

    render_text(
        canvas,
        font,
        bright_green,
        20,
        140,
        format!("Lines Cleared: {0}", game.score_lines_cleared),
    );

    render_text(
        canvas,
        font,
        bright_green,
        20,
        200,
        format!("Score: {0}", game.score_points),
    );

    if game.is_gameover() {
        render_text_centered(
            canvas,
            font,
            bright_red,
            (window_width / 2) as i32,
            50,
            "GAME OVER!".to_string(),
        )
    };
}

#[derive(PartialEq)]
enum Mode {
    Tetris,
    Replay,
}

fn load_last_game_state() -> Result<game::Game, String> {
    if let Ok(last_game_state_file) = fs::File::open("last_game_state.json") {
        let last_game_state_reader = io::BufReader::new(last_game_state_file);

        let last_game_state: game::Game =
            serde_json::from_reader(last_game_state_reader).map_err(|e| e.to_string())?;
        if !last_game_state.is_gameover() {
            return Ok(last_game_state);
        }
    }

    return Err("Previous game state not available.".to_string());
}

fn main() -> Result<(), String> {
    let mut ui_layers = UI_LAYER_GAME | UI_LAYER_CONSOLE;
    println!("ui_layers {0}", ui_layers);
    let mut console = Console::new();

    let prefs = preferences::Preferences::new();
    let mut paused = false;

    let mut mode = Mode::Tetris;
    let args: Vec<String> = env::args().collect();

    let mut replay: Option<Replay> = None;

    let mut replay_action_index = 0;
    let mut last_game = None;
    if args.len() > 1 {
        let recording_file = fs::File::open(args[1].clone()).map_err(|e| e.to_string())?;
        let recording_file_reader = io::BufReader::new(recording_file);
        let recording =
            serde_json::from_reader(recording_file_reader).map_err(|e| e.to_string())?;
        replay = Some(Replay { recording });
        mode = Mode::Replay;
    } else {
        match load_last_game_state() {
            Ok(lgs) => last_game = Some(lgs),
            Err(e) => println!("{}", e),
        }
    }

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut font = ttf_context.load_font(
        "/usr/share/fonts/adobe-source-code-pro/SourceCodePro-Regular.otf",
        48,
    )?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let _smaller_font =
        ttf_context.load_font("/usr/share/fonts/TTF/PressStart2P-Regular.ttf", 18)?;

    println!(
        "video driver = {:?}, display name = {:?}",
        video_subsys.current_video_driver(),
        video_subsys.display_name(0)
    );

    let initial_window_width = 1920;
    let initial_window_height = 1080;

    let window = video_subsys
        .window("Tetris", initial_window_width, initial_window_height)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    // println!("{:?} {:?}", L, T);

    let _total = 0;
    let mut frames = 0;
    let mut slowest_frame = 0.0;

    let mut t: f64 = 0.0;
    let dt: f64 = 1.0 / 250.0; // 4ms tick rate.

    let game_loop_start_at = time::Instant::now();
    let mut start_time = time::Instant::now();
    let mut accumulator: f64 = 0.0;

    let mut game_rules = game::Rules::new();
    game_rules.lock_delay(50);
    // game_rules.lock_delay_on_hard_drop(true);
    let mut game_ticks = 0;

    let mut game = match replay {
        Some(ref r) => {
            let replay_pieces = ReplayPieces::new(r);
            game::Game::new(game_rules.clone(), Some(Box::new(replay_pieces)))?
        }
        None => {
            if let Some(g) = last_game {
                paused = true;
                g
            } else {
                game::Game::new(game_rules.clone(), None)?
            }
        }
    };

    'main: loop {
        frames = frames + 1;
        let now = time::Instant::now();
        let mut frame_time = now - start_time;
        let frame_rate = 1000000.0 / frame_time.as_micros() as f64;
        // println!("frames = {:?}, frame time = {:?}, frame rate = {:?}", frames, frame_time, frame_rate);
        //
        let ftf = frame_time.as_secs_f64();
        if ftf > slowest_frame {
            println!("SLOWEST frame so far frame={0}, duration={1}", frames, ftf);
            slowest_frame = ftf;
        }
        if ftf > 0.25 {
            println!("******************************************************* SLOW");
            frame_time = time::Duration::from_millis(250);
        }
        start_time = now;

        if ui_layers & UI_LAYER_CONSOLE == UI_LAYER_CONSOLE {
            let ui_actions = console.process_events(&mut events);
            for action in ui_actions.iter() {
                match *action {
                    UIAction::Quit => break 'main,
                    UIAction::HideConsole => ui_layers = ui_layers ^ UI_LAYER_CONSOLE,
                }
            }
        }

        for event in events.poll_iter() {
            match event {
                event::Event::Quit { .. } => break 'main,
                event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    keyboard::Keycode::Escape => break 'main,
                    keyboard::Keycode::Backquote => ui_layers = ui_layers | UI_LAYER_CONSOLE,
                    keyboard::Keycode::Space => {
                        if game.is_gameover() {
                            game = game::Game::new(game_rules.clone(), None)?;
                            mode = Mode::Tetris;
                        } else if paused {
                            paused = false
                        } else {
                            paused = true;
                        }
                    }

                    _ => {
                        if !paused && !game.is_gameover() && mode == Mode::Tetris {
                            match keycode {
                                keyboard::Keycode::Kp7 => {
                                    if let Err(e) = game.queue_action(actions::Action::MoveLeft) {
                                        println!("Dropped GAME ACTION {:?}", e);
                                    }
                                }
                                keyboard::Keycode::Kp9 => {
                                    let _ = game.queue_action(actions::Action::MoveRight);
                                }
                                keyboard::Keycode::Kp4 => {
                                    let _ = game.queue_action(actions::Action::Drop);
                                }
                                keyboard::Keycode::Kp5 => {
                                    let _ = game.queue_action(actions::Action::MoveDown);
                                }
                                keyboard::Keycode::Kp8 => {
                                    let _ = game.queue_action(actions::Action::Rotate);
                                }
                                keyboard::Keycode::KpPlus => game.speed_up(),
                                keyboard::Keycode::KpMinus => game.speed_down(),
                                _ => (),
                            }
                        }
                    }
                },

                event::Event::MouseButtonDown { .. } => {}

                _ => {}
            }
        }

        if !paused && !game.is_gameover() {
            accumulator = accumulator + frame_time.as_secs_f64();
        }

        let mut acc_runs = 0;
        if !paused && !game.is_gameover() {
            while accumulator >= dt {
                acc_runs += 1;
                t += dt;
                accumulator -= dt;

                if mode == Mode::Replay {
                    if let Some(ref r) = replay {
                        while replay_action_index < r.recording.events.len() - 1
                            && !matches!(
                                r.recording.events[replay_action_index].kind,
                                game::recordings::EventKind::Action(_)
                            )
                        {
                            replay_action_index += 1
                        }
                        match r.recording.events[replay_action_index].kind {
                            game::recordings::EventKind::Action(a) => {
                                if r.recording.events[replay_action_index].at <= game_ticks {
                                    replay_action_index += 1;
                                    let _ = game.queue_action(a);
                                }
                            }
                            _ => (),
                        }
                    }
                }

                game_ticks = game.tick();
            }
        }

        if acc_runs > 1 {
            println!("Multiple({acc_runs}) simulations during single drawing frame.");
        }

        render_game(&mut canvas, &mut game, &prefs, &font);
        if ui_layers & UI_LAYER_CONSOLE == UI_LAYER_CONSOLE {
            console.render(&mut canvas);
        }

        if paused {
            let x: i32 = (canvas.window().size().0 / 2) as i32;

            render_text_centered(
                &mut canvas,
                &font,
                pixels::Color::RGBA(255, 0, 0, 255),
                x,
                50,
                "PAUSED...".to_string(),
            )
        };

        render_text(
            &mut canvas,
            &font,
            pixels::Color::RGB(0, 0, 255),
            20,
            20,
            format!("{:.2} fps", frame_rate),
        );

        canvas.present()
    }

    let run_time = time::Instant::now().duration_since(game_loop_start_at);
    println!("Total run time = {:?}", run_time.as_secs());
    println!("Total frames rendered = {0}", frames);
    let mut run_time_secs = run_time.as_secs();
    if run_time_secs < 1 {
        run_time_secs = 1
    }
    println!("FPS = {0}", frames / run_time_secs);

    if mode != Mode::Replay {
        let mut recording_file =
            fs::File::create("last_game_recording.json").map_err(|e| e.to_string())?;
        let _ = serde_json::to_writer_pretty(&mut recording_file, &game.recording)
            .map_err(|e| e.to_string())?;
    }

    let mut last_game_state_file =
        fs::File::create("last_game_state.json").map_err(|e| e.to_string())?;
    let _ = serde_json::to_writer_pretty(&mut last_game_state_file, &game)
        .map_err(|e| e.to_string())?;

    Ok(())
}
