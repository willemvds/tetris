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

pub struct Menu<'ttf, 'rwops> {
    font: ttf::Font<'ttf, 'rwops>,
}

impl<'ttf, 'rwops> Menu<'ttf, 'rwops> {
    pub fn new(
        registry: &'rwops assets::Registry,
        ttf_context: &'ttf ttf::Sdl2TtfContext,
    ) -> Result<Menu<'ttf, 'rwops>, String> {
        let asset = registry
            .get("PressStart2P-Regular.ttf")
            .map_err(|e| e.to_string())?;
        {
            let rw = rwops::RWops::from_bytes(asset)?;
            let font = ttf_context.load_font_from_rwops(rw, 34)?;
            return Ok(Menu { font });
        }
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        let (canvas_width, canvas_height) = canvas.window().size();
        canvas.set_draw_color(pixels::Color::RGBA(200, 100, 13, 100));
        let _ = canvas.fill_rect(rect::Rect::new(0, 0, canvas_width, canvas_height));

        canvas.set_draw_color(pixels::Color::RGBA(50, 220, 20, 255));
        let _ = canvas.fill_rect(rect::Rect::new(
            (canvas_width / 2) as i32 - 200,
            100,
            400,
            100,
        ));

        render_text(
            canvas,
            &self.font,
            pixels::Color::RGBA(255, 255, 255, 255),
            100,
            500,
            &"Play".to_string(),
        );

        render_text(
            canvas,
            &self.font,
            pixels::Color::RGBA(255, 255, 255, 255),
            100,
            600,
            &"Quit".to_string(),
        );
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
                    keyboard::Keycode::Escape => ui_actions.push(actions::Action::MenuHide),
                    keyboard::Keycode::Backquote => ui_actions.push(actions::Action::ConsoleShow),
                    _ => (),
                },
                _ => (),
            }
        }

        ui_actions
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
