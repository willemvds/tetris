use crate::actions;
use crate::assets;

use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::ttf;
use sdl2::video;

pub struct Console<'ttf, 'rwops> {
    history: Vec<String>,
    buffer: String,

    font: ttf::Font<'ttf, 'rwops>,
}

impl<'ttf, 'rwops> Console<'ttf, 'rwops> {
    pub fn new(
        registry: &'rwops assets::Registry,
        ttf_context: &'ttf ttf::Sdl2TtfContext,
    ) -> Result<Console<'ttf, 'rwops>, String> {
        let font_bytes = registry
            .get("PressStart2P-Regular.ttf")
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
                    scancode: Some(scancode),
                    ..
                } => match keycode {
                    keyboard::Keycode::Backquote => ui_actions.push(actions::Action::ConsoleHide),
                    keyboard::Keycode::Backspace => {
                        self.buffer.pop();
                    }
                    keyboard::Keycode::Return => {
                        if self.buffer.len() > 0 {
                            let cmd = self.buffer.clone();
                            ui_actions.push(actions::Action::ConsoleCommand(cmd.clone()));

                            self.println(format!("> {0}", cmd));
                            self.buffer = "".to_string();
                        }
                    }
                    _ => {
                        let keynum = keycode as u8;
                        if (keynum >= 97 && keynum <= 122) || keynum == 32 {
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

    pub fn println(&mut self, text: String) {
        self.history.push(text)
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        let (canvas_width, _) = canvas.window().size();
        canvas.set_draw_color(pixels::Color::RGBA(80, 80, 80, 200));
        let _ = canvas.fill_rect(rect::Rect::new(0, 0, canvas_width, 500));

        let mut y = 10;
        for line in self.history.iter() {
            render_text(
                canvas,
                &self.font,
                pixels::Color::RGBA(255, 255, 255, 255),
                20,
                y,
                line,
            );
            y += 30
        }

        if self.buffer.len() > 0 {
            render_text(
                canvas,
                &self.font,
                pixels::Color::RGBA(0, 255, 0, 255),
                20,
                450,
                &self.buffer,
            );
        }
    }
}

fn render_text(
    canvas: &mut render::Canvas<video::Window>,
    font: &ttf::Font,
    colour: pixels::Color,
    x: i32,
    y: i32,
    text: &String,
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
