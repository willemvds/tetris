use std::collections;
use std::ops;
use std::time;

use crate::actions;
use crate::assets;
use crate::graphics;
use crate::preferences;
use crate::replays;
use crate::tetris;
use crate::tetris::game;
use crate::tetris::playfield;
use crate::tetris::recordings;
use crate::tetris::tetrominos;

use sdl2::controller;
use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::ttf;
use sdl2::video;

#[derive(PartialEq)]
enum Mode {
    Tetris,
    Replay,
}

// invariants:
// - There is always a game instance locked and loaded.
pub struct GameShell<'ttf, 'rwops> {
    game: game::Game,
    game_ticks: usize,
    paused: bool,
    mode: Mode,
    replay: Option<replays::Replay>,
    replay_action_index: usize,

    accumulator: f64,

    score_label_font: ttf::Font<'ttf, 'rwops>,
    score_value_font: ttf::Font<'ttf, 'rwops>,

    keymap: collections::HashMap<keyboard::Keycode, actions::Action>,
    controller_x: i16,
    controller_trigger: i16,
}

impl<'ttf, 'rwops> GameShell<'ttf, 'rwops> {
    pub fn new(
        initial_game: game::Game,
        registry: &'rwops assets::Registry,
        ttf_context: &'ttf ttf::Sdl2TtfContext,
    ) -> Result<GameShell<'ttf, 'rwops>, String> {
        let scp_font_bytes = registry
            .get("fonts/SourceCodePro-Regular.otf")
            .map_err(|e| e.to_string())?;
        let ps2_font_bytes = registry
            .get("fonts/PressStart2P-Regular.ttf")
            .map_err(|e| e.to_string())?;

        let score_label_rwops = rwops::RWops::from_bytes(scp_font_bytes)?;
        let score_label_font = ttf_context.load_font_from_rwops(score_label_rwops, 28)?;
        let score_value_rwops = rwops::RWops::from_bytes(ps2_font_bytes)?;
        let mut score_value_font = ttf_context.load_font_from_rwops(score_value_rwops, 44)?;
        score_value_font.set_style(sdl2::ttf::FontStyle::BOLD);

        let keymap = collections::HashMap::from([(
            keyboard::Keycode::F11,
            actions::Action::ToggleFullScreen,
        )]);

        Ok(GameShell {
            game: initial_game,
            game_ticks: 0,
            paused: true,
            mode: Mode::Tetris,
            replay: None,
            replay_action_index: 0,

            accumulator: 0.0,

            score_label_font,
            score_value_font,

            keymap,
            controller_x: 0,
            controller_trigger: 0,
        })
    }

    pub fn load_replay(&mut self, gm: game::Game, replay: replays::Replay) {
        self.game = gm;
        self.game_ticks = 0;
        self.paused = true;
        self.mode = Mode::Replay;
        self.replay = Some(replay);
        self.replay_action_index = 0;
    }

    pub fn load_game(&mut self, game: game::Game) {
        self.game = game
    }

    pub fn pause(&mut self) {
        self.paused = true
    }

    pub fn unpause(&mut self) {
        self.paused = false
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_gameover(&self) -> bool {
        self.game.is_gameover()
    }

    pub fn is_showing_replay(&self) -> bool {
        self.mode == Mode::Replay
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }

    pub fn recording(&self) -> Result<&recordings::Recording, String> {
        match self.mode {
            Mode::Replay => Err("Sorry, don't have a recording for you.".to_string()),
            Mode::Tetris => {
                if self.game.is_gameover() {
                    Ok(&self.game.recording)
                } else {
                    Err("Recording is not available while game is in progress.".to_string())
                }
            }
        }
    }

    pub fn game(&self) -> &game::Game {
        &self.game
    }

    pub fn frame_tick(&mut self, frame_time: time::Duration, dt: f64) {
        if self.is_paused() {
            return;
        }

        if self.game.is_gameover() {
            return;
        }

        self.accumulator += frame_time.as_secs_f64();

        let mut acc_runs = 0;
        while self.accumulator >= dt {
            acc_runs += 1;
            self.accumulator -= dt;

            if self.mode == Mode::Replay {
                if let Some(ref r) = self.replay {
                    while self.replay_action_index < r.recording.events.len() - 1
                        && !matches!(
                            r.recording.events[self.replay_action_index].kind,
                            tetris::recordings::EventKind::Action(_)
                        )
                    {
                        self.replay_action_index += 1
                    }
                    if let tetris::recordings::EventKind::Action(a) =
                        r.recording.events[self.replay_action_index].kind
                    {
                        if r.recording.events[self.replay_action_index].at <= self.game_ticks {
                            self.replay_action_index += 1;
                            let _ = self.game.queue_action(a);
                        }
                    }
                }
            }

            self.game_ticks = self.game.tick();
        }
    }

    pub fn process_events(&mut self, event_pump: &mut sdl2::EventPump) -> Vec<actions::Action> {
        let mut ui_actions = vec![];

        for event in event_pump.poll_iter() {
            match event {
                event::Event::Quit { .. } => ui_actions.push(actions::Action::Quit),
                event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    //                    keyboard::Keycode::F11 => ui_actions.push(actions::Action::ToggleFullScreen),
                    keyboard::Keycode::Escape => ui_actions.push(actions::Action::MenuShow),
                    keyboard::Keycode::Backquote => ui_actions.push(actions::Action::ConsoleShow),
                    keyboard::Keycode::Space => {
                        if self.is_gameover() {
                            ui_actions.push(actions::Action::NewGame)
                        } else {
                            ui_actions.push(actions::Action::TogglePause)
                        }
                    }

                    _ => {
                        if let Some(action) = self.keymap.get(&keycode) {
                            ui_actions.push(action.clone());
                        }

                        if !self.paused && !self.game.is_gameover() && self.mode == Mode::Tetris {
                            match keycode {
                                keyboard::Keycode::Kp7 => {
                                    let _ =
                                        self.game.queue_action(tetris::actions::Action::MoveLeft);
                                }
                                keyboard::Keycode::Kp9 => {
                                    let _ =
                                        self.game.queue_action(tetris::actions::Action::MoveRight);
                                }
                                keyboard::Keycode::Kp4 => {
                                    let _ = self.game.queue_action(tetris::actions::Action::Drop);
                                }
                                keyboard::Keycode::Kp5 => {
                                    let _ =
                                        self.game.queue_action(tetris::actions::Action::MoveDown);
                                }
                                keyboard::Keycode::Kp8 => {
                                    let _ = self.game.queue_action(tetris::actions::Action::Rotate);
                                }
                                _ => (),
                            }
                        }
                    }
                },
                event::Event::ControllerButtonDown { button, .. } => match button {
                    controller::Button::Back => ui_actions.push(actions::Action::MenuShow),
                    controller::Button::Start => {
                        if self.is_gameover() {
                            ui_actions.push(actions::Action::NewGame)
                        } else {
                            ui_actions.push(actions::Action::TogglePause)
                        }
                    }
                    controller::Button::X => {
                        let _ = self.game.queue_action(tetris::actions::Action::MoveLeft);
                    }
                    controller::Button::B => {
                        let _ = self.game.queue_action(tetris::actions::Action::MoveRight);
                    }
                    controller::Button::A => {
                        let _ = self.game.queue_action(tetris::actions::Action::MoveDown);
                    }
                    controller::Button::Y => {
                        let _ = self.game.queue_action(tetris::actions::Action::Rotate);
                    }
                    _ => println!("Controller Button = {:?}", button),
                },
                event::Event::ControllerAxisMotion {
                    axis: controller::Axis::TriggerRight,
                    value: val,
                    ..
                } => {
                    self.controller_trigger = val;
                }
                event::Event::ControllerAxisMotion {
                    axis: controller::Axis::LeftX,
                    value: val,
                    ..
                } => {
                    self.controller_x = val;
                }
                _ => {}
            }
        }

        if self.controller_trigger > 30000 {
            let _ = self.game.queue_action(tetris::actions::Action::Drop);
        }

        if self.controller_x > 15000 {
            let _ = self.game.queue_action(tetris::actions::Action::MoveRight);
        } else if self.controller_x < -15000 {
            let _ = self.game.queue_action(tetris::actions::Action::MoveLeft);
        }

        ui_actions
    }

    pub fn render(
        &mut self,
        canvas: &mut render::Canvas<video::Window>,
        prefs: &preferences::Preferences,
    ) {
        let label_font = &self.score_label_font;
        let value_font = &self.score_value_font;

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        let (window_width, window_height) = canvas.window().size();
        let cell_size: i32 = (window_height / 30) as i32;

        draw_game(canvas, &self.game, cell_size);

        let start_x: i32 = (window_width as i32 / 2)
            - (cell_size * self.game.play_field.cols as i32 / 2)
            - (3 * cell_size);
        let start_y: i32 = 1;

        if self.game.piece.y < 4 {
            draw_partial_shape(
                canvas,
                *self.game.piece.form(),
                4 - self.game.piece.y as i16,
                tetromino_colour(self.game.piece.tetromino),
                cell_size,
                start_x + (self.game.piece.x as i32 * cell_size),
                start_y + (self.game.piece.y as i32 * cell_size),
            );
        } else {
            draw_shape(
                canvas,
                *self.game.piece.form(),
                tetromino_colour(self.game.piece.tetromino),
                cell_size,
                start_x + (self.game.piece.x as i32 * cell_size),
                start_y + (self.game.piece.y as i32 * cell_size),
            );
        }

        if self.game.drop_distance() > 0 {
            if prefs.drop_indicator == preferences::DropIndicatorStyle::Outline {
                draw_shape_outline(
                    canvas,
                    *self.game.piece.form(),
                    tetromino_colour(self.game.piece.tetromino),
                    cell_size,
                    start_x + (self.game.piece.x as i32 * cell_size),
                    start_y
                        + (self.game.piece.y + self.game.drop_distance() as u16 - 1) as i32
                            * cell_size,
                )
            } else if prefs.drop_indicator == preferences::DropIndicatorStyle::Triangles {
                draw_shape_triangles(
                    canvas,
                    *self.game.piece.form(),
                    tetromino_colour(self.game.piece.tetromino),
                    cell_size,
                    start_x + (self.game.piece.x as i32 * cell_size),
                    start_y
                        + (self.game.piece.y + self.game.drop_distance() as u16 - 1) as i32
                            * cell_size,
                )
            }
        }

        draw_shape(
            canvas,
            tetrominos::from_kind(self.game.next_piece).forms[0],
            tetromino_colour(self.game.next_piece),
            cell_size,
            start_x + (self.game.play_field.cols as i32 * cell_size) + (window_width as i32 / 10),
            start_y + (window_width as i32 / 10),
        );

        let bright_red = pixels::Color::RGBA(255, 0, 0, 255);
        let label_colour = pixels::Color::RGBA(255, 255, 255, 255);
        let value_colour = pixels::Color::RGB(0, 255, 0);

        let width_third = window_width / 3;
        graphics::render_text(
            canvas,
            label_font,
            label_colour,
            (2 * width_third) as i32 - 120,
            500,
            "Level",
        );

        graphics::render_text(
            canvas,
            value_font,
            bright_red,
            (2 * width_third) as i32 - 120,
            550,
            &format!("{0}", self.game.level),
        );

        graphics::render_text(
            canvas,
            label_font,
            label_colour,
            (2 * width_third) as i32 - 120,
            620,
            "Lines",
        );

        graphics::render_text(
            canvas,
            value_font,
            value_colour,
            (2 * width_third) as i32 - 120,
            670,
            &format!("{0}", self.game.score_lines_cleared),
        );

        graphics::render_text(
            canvas,
            label_font,
            label_colour,
            (2 * width_third) as i32 - 120,
            740,
            "Score",
        );

        graphics::render_text(
            canvas,
            value_font,
            value_colour,
            (2 * width_third) as i32 - 120,
            790,
            &format!("{0}", self.game.score_points),
        );

        if self.game.is_gameover() {
            graphics::render_text_centered(
                canvas,
                value_font,
                bright_red,
                (window_width / 2) as i32,
                50,
                "GAME OVER!",
            )
        } else if self.paused {
            let x: i32 = (canvas.window().size().0 / 2) as i32;

            graphics::render_text_centered(
                canvas,
                value_font,
                pixels::Color::RGBA(255, 0, 0, 255),
                x,
                50,
                "PAUSED...",
            )
        };
    }
}

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
            canvas.set_draw_color(pf.matrix[row][col].color());
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
