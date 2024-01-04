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

    let start_x: i32 = (canvas_width as i32 - (size * 10)) / 2;
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
    text: String,
    x: i32,
    y: i32,
) {
    let texture_creator = canvas.texture_creator();

    let (char_width, char_height) = font.size_of_char('C').unwrap();

    let surface = font
        .render(&text)
        .blended(pixels::Color::RGBA(0, 255, 0, 255))
        .map_err(|e| e.to_string())
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let target = rect::Rect::new(x, y, char_width * text.len() as u32, char_height);

    let _ = canvas.copy(&texture, None, Some(target));
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut font = ttf_context.load_font(
        "/usr/share/fonts/adobe-source-code-pro/SourceCodePro-Regular.otf",
        48,
    )?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let smaller_font =
        ttf_context.load_font("/usr/share/fonts/TTF/PressStart2P-Regular.ttf", 18)?;

    println!(
        "video driver = {:?}, display name = {:?}",
        video_subsys.current_video_driver(),
        video_subsys.display_name(0)
    );

    let initial_window_width = 1920;
    let initial_window_height = 1080;

    let window = video_subsys
        .window("Panda Tetris", initial_window_width, initial_window_height)
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

    let mut t: f64 = 0.0;
    let dt: f64 = 1.0 / 240.0;

    let game_loop_start_at = time::Instant::now();
    let mut start_time = time::Instant::now();
    let mut accumulator: f64 = 0.0;

    let mut game = game::Game::new()?;

    'main: loop {
        frames = frames + 1;

        let now = time::Instant::now();
        let mut frame_time = now - start_time;
        let frame_rate = 1000000.0 / frame_time.as_micros() as f64;
        // println!("frames = {:?}, frame time = {:?}, frame rate = {:?}", frames, frame_time, frame_rate);
        if frame_time.as_secs_f64() > 0.25 {
            println!("******************************************************* SLOW");
            frame_time = time::Duration::from_millis(250);
        }
        start_time = now;
        accumulator = accumulator + frame_time.as_secs_f64();

        let mut acc_runs = 0;
        while accumulator >= dt {
            acc_runs += 1;
            t += dt;
            accumulator -= dt;

            for event in events.poll_iter() {
                match event {
                    event::Event::Quit { .. } => break 'main,

                    event::Event::Window { win_event: wev, .. } => match wev {
                        event::WindowEvent::SizeChanged(new_width, new_height) => {
                            println!("New window width={0} height={1}", new_width, new_height);
                        }
                        _ => (),
                    },

                    event::Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => match keycode {
                        keyboard::Keycode::Escape => break 'main,
                        keyboard::Keycode::Space => {
                            if game.is_playing() {
                                game.pause()
                            } else {
                                game.unpause()
                            }
                        }

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
                    },

                    event::Event::MouseButtonDown { .. } => {}

                    _ => {}
                }
            }

            if game.is_playing() {
                game.sim(t, dt, accumulator);
            }
        }

        if acc_runs > 1 {
            println!("Multiple({acc_runs}) simulations during single frame.");
        }

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        let (window_width, window_height) = canvas.window().size();
        let cell_size: i32 = (window_height / 30) as i32;

        draw_game(&mut canvas, &game, cell_size);

        thread::sleep(time::Duration::from_millis(1));

        let start_x: i32 = (window_width as i32 - (cell_size * 10)) / 2;
        let start_y: i32 = 1;

        if game.piece.y < 4 {
            draw_partial_shape(
                &mut canvas,
                *game.piece.form(),
                4 - game.piece.y as i16,
                tetromino_colour(game.piece.tetromino.kind),
                cell_size,
                start_x + (game.piece.x as i32 * cell_size),
                start_y + (game.piece.y as i32 * cell_size),
            );
        } else {
            draw_shape(
                &mut canvas,
                *game.piece.form(),
                tetromino_colour(game.piece.tetromino.kind),
                cell_size,
                start_x + (game.piece.x as i32 * cell_size),
                start_y + (game.piece.y as i32 * cell_size),
            );
        }

        if game.drop_distance() > 0 {
            draw_shape_outline(
                &mut canvas,
                *game.piece.form(),
                tetromino_colour(game.piece.tetromino.kind),
                cell_size,
                start_x + (game.piece.x as i32 * cell_size),
                start_y + (game.piece.y + game.drop_distance() as u16 - 1) as i32 * cell_size,
            )
        }

        draw_shape(
            &mut canvas,
            game.next_piece.forms[0],
            tetromino_colour(game.next_piece.kind),
            cell_size,
            1300,
            160,
        );

        render_text(&mut canvas, &font, format!("{:.2} fps", frame_rate), 20, 20);

        render_text(
            &mut canvas,
            &font,
            format!("Lines Cleared: {0}", game.score_lines_cleared),
            20,
            140,
        );

        render_text(
            &mut canvas,
            &font,
            format!("Score: {0}", game.score_points),
            20,
            200,
        );

        render_text(
            &mut canvas,
            &smaller_font,
            "Next Piece:".to_string(),
            1300,
            100,
        );

        canvas.present();
    }

    let run_time = time::Instant::now().duration_since(game_loop_start_at);
    println!("Total run time = {:?}", run_time.as_secs());
    println!("Total frames rendered = {0}", frames);
    println!("FPS = {0}", frames / run_time.as_secs());

    Ok(())
}
