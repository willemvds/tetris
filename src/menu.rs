use crate::actions;
use crate::assets;
use crate::graphics;

use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::ttf;
use sdl2::video;

pub enum MenuOptionSize {
    Regular,
    Large,
}

pub struct MenuOption {
    text: String,
    size: MenuOptionSize,
    handler_fn: fn() -> actions::Action,
}

impl MenuOption {
    fn new(text: String, size: MenuOptionSize, handler_fn: fn() -> actions::Action) -> MenuOption {
        MenuOption {
            text,
            size,
            handler_fn,
        }
    }
}

pub struct Menu<'ttf, 'rwops> {
    regular_font: ttf::Font<'ttf, 'rwops>,
    large_font: ttf::Font<'ttf, 'rwops>,

    options: Vec<MenuOption>,
    selected_option: Option<usize>,
}

fn quit_action() -> actions::Action {
    actions::Action::Quit
}

fn play_action() -> actions::Action {
    actions::Action::Play
}

impl<'ttf, 'rwops> Menu<'ttf, 'rwops> {
    pub fn new(
        registry: &'rwops assets::Registry,
        ttf_context: &'ttf ttf::Sdl2TtfContext,
    ) -> Result<Menu<'ttf, 'rwops>, String> {
        let font_bytes = registry
            .get("fonts/PressStart2P-Regular.ttf")
            .map_err(|e| e.to_string())?;
        {
            let regular_rwops = rwops::RWops::from_bytes(font_bytes)?;
            let regular_font = ttf_context.load_font_from_rwops(regular_rwops, 28)?;

            let large_rwops = rwops::RWops::from_bytes(font_bytes)?;
            let large_font = ttf_context.load_font_from_rwops(large_rwops, 42)?;

            let mut menu = Menu {
                regular_font,
                large_font,
                options: vec![],
                selected_option: None,
            };

            menu.options.push(MenuOption::new(
                "Play".to_string(),
                MenuOptionSize::Large,
                play_action,
            ));
            menu.options.push(MenuOption::new(
                "Replays".to_string(),
                MenuOptionSize::Regular,
                play_action,
            ));

            menu.options.push(MenuOption::new(
                "Quit (q)".to_string(),
                MenuOptionSize::Regular,
                quit_action,
            ));
            menu.selected_option = Some(0);

            Ok(menu)
        }
    }

    pub fn render_option(
        &self,
        canvas: &mut render::Canvas<video::Window>,
        opt: &MenuOption,
        y: i32,
        selected: bool,
    ) {
        if selected {
            canvas.set_draw_color(pixels::Color::RGBA(252, 252, 252, 255));
            let _ = canvas.fill_rect(rect::Rect::new(50, y - 20, 500, 80));
        }
        let f = match opt.size {
            MenuOptionSize::Regular => &self.regular_font,
            MenuOptionSize::Large => &self.large_font,
        };
        let c = match selected {
            true => pixels::Color::RGBA(0, 255, 0, 255),
            false => pixels::Color::RGBA(255, 255, 255, 255),
        };
        graphics::render_text(canvas, f, c, 100, y, &opt.text);
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        let (canvas_width, canvas_height) = canvas.window().size();
        canvas.set_draw_color(pixels::Color::RGBA(200, 100, 13, 100));
        let _ = canvas.fill_rect(rect::Rect::new(
            0,
            0,
            canvas_width / 3,
            canvas_height,
        ));

        let mut y = 500;
        for (idx, opt) in self.options.iter().enumerate() {
            self.render_option(
                canvas,
                opt,
                y,
                idx == self.selected_option.unwrap_or_default(),
            );
            y += 100;
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
                    keyboard::Keycode::Escape => ui_actions.push(actions::Action::MenuHide),
                    keyboard::Keycode::Backquote => ui_actions.push(actions::Action::ConsoleShow),
                    keyboard::Keycode::Q => ui_actions.push(actions::Action::Quit),
                    keyboard::Keycode::Down => self.move_down(),
                    keyboard::Keycode::Up => self.move_up(),
                    keyboard::Keycode::Return => self.action(&mut ui_actions),
                    _ => (),
                },
                _ => (),
            }
        }

        ui_actions
    }

    fn move_down(&mut self) {
        match self.selected_option {
            Some(option) => {
                if option < self.options.len() - 1 {
                    self.selected_option = Some(option + 1)
                }
            }
            None => {
                self.selected_option = Some(self.options.len() - 1);
            }
        }
    }

    fn move_up(&mut self) {
        match self.selected_option {
            Some(option) => {
                if option > 0 {
                    self.selected_option = Some(option - 1)
                }
            }
            None => {
                self.selected_option = Some(0);
            }
        }
    }

    fn action(&mut self, ui_actions: &mut Vec<actions::Action>) {
        match self.selected_option {
            None => (),
            Some(option) => ui_actions.push((self.options[option].handler_fn)()),
        }
    }
}
