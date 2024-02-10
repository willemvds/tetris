use crate::actions;
use crate::assets;
use crate::graphics;
use crate::tetris::tetrominos;

use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::ttf;
use sdl2::video;

enum ConsoleBlock {
    Text(pixels::Color, String),
    Tetromino(tetrominos::Kind),
}

impl ConsoleBlock {
    fn render(&self, canvas: &mut render::Canvas<video::Window>, font: &ttf::Font, y: i32) -> u32 {
        match self {
            ConsoleBlock::Text(colour, text) => {
                graphics::render_text(canvas, font, colour.clone(), 20, y, text);
                40
            }
            ConsoleBlock::Tetromino(kind) => {
                let tetro = tetrominos::from_kind(*kind);

                let cell_size = 32;
                for i in 0..4 {
                    let form = tetro.forms[i];
                    graphics::render_form(
                        canvas,
                        form,
                        graphics::tetromino_colour(*kind).clone(),
                        cell_size,
                        20 + ((cell_size * 4) * i as i32) + (cell_size * i as i32),
                        y - (cell_size * 3),
                    );
                }

                cell_size as u32 * 4
            }
        }
    }
}

pub struct Console<'ttf, 'rwops> {
    history: Vec<ConsoleBlock>,
    buffer: String,

    font: ttf::Font<'ttf, 'rwops>,
}

impl<'ttf, 'rwops> Console<'ttf, 'rwops> {
    pub fn new(
        registry: &'rwops assets::Registry,
        ttf_context: &'ttf ttf::Sdl2TtfContext,
    ) -> Result<Console<'ttf, 'rwops>, String> {
        let font_bytes = registry
            .get("fonts/SourceCodePro-Regular.otf")
            .map_err(|e| e.to_string())?;
        let rw = rwops::RWops::from_bytes(font_bytes)?;
        let font = ttf_context.load_font_from_rwops(rw, 18)?;

        Ok(Console {
            history: vec![],
            buffer: "".to_string(),
            font,
        })
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
                    keyboard::Keycode::Backquote => ui_actions.push(actions::Action::ConsoleHide),
                    keyboard::Keycode::Backspace => {
                        self.buffer.pop();
                    }
                    keyboard::Keycode::Return => {
                        if !self.buffer.is_empty() {
                            let cmd = self.buffer.clone();
                            ui_actions.push(actions::Action::ConsoleCommand(cmd.clone()));

                            self.println_with_colour(
                                pixels::Color::RGBA(0, 255, 0, 255),
                                format!("> {0}", cmd),
                            );
                            self.buffer = "".to_string();
                        }
                    }
                    _ => {
                        let keynum = keycode as u8;
                        if (97..=122).contains(&keynum) || keynum == 32 {
                            self.buffer.push(keycode as u8 as char);
                        } else {
                            println!("{0}", keycode as i32);
                            println!("Console got this keycode: {0}", keycode);
                        }
                    }
                },
                _ => (),
            }
        }

        ui_actions
    }

    pub fn println_with_colour(&mut self, colour: pixels::Color, text: String) {
        self.history.push(ConsoleBlock::Text(colour, text))
    }

    pub fn println(&mut self, text: String) {
        self.history.push(ConsoleBlock::Text(
            pixels::Color::RGBA(255, 255, 255, 0),
            text,
        ))
    }

    pub fn print_tetromino(&mut self, kind: tetrominos::Kind) {
        self.history.push(ConsoleBlock::Tetromino(kind))
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        let (canvas_width, canvas_height) = canvas.window().size();
        let height_third = canvas_height / 3;
        canvas.set_draw_color(pixels::Color::RGBA(80, 80, 80, 200));
        let _ = canvas.fill_rect(rect::Rect::new(0, 0, canvas_width, height_third * 2));

        canvas.set_draw_color(pixels::Color::RGB(200, 0, 0));
        let _ = canvas.fill_rect(rect::Rect::new(
            0,
            (height_third * 2) as i32 - 3,
            canvas_width,
            3,
        ));

        let mut y = (height_third as i32 * 2) - 80;
        for block in self.history.iter().rev() {
            y -= block.render(canvas, &self.font, y) as i32;
        }

        if !self.buffer.is_empty() {
            graphics::render_text(
                canvas,
                &self.font,
                pixels::Color::RGBA(0, 255, 0, 255),
                20,
                (height_third * 2) as i32 - 50,
                &self.buffer,
            );
        }
    }
}
