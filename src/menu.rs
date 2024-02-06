use crate::actions;
use crate::assets;
use crate::graphics;
use crate::preferences;

use sdl2::event;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::ttf;
use sdl2::video;

enum MenuAction {
    ShowPreferences,
}

enum SelectionAction {
    UI(actions::Action),
    Menu(MenuAction),
}

struct RadioOption {
    text: String,
}

impl RadioOption {
    fn new(text: String) -> RadioOption {
        RadioOption { text }
    }

    fn render(&self, canvas: &mut render::Canvas<video::Window>, font: &ttf::Font, x: i32, y: i32) {
        let text_colour = pixels::Color::RGBA(255, 255, 255, 255);

        graphics::render_text(canvas, font, text_colour, x, y, &self.text);
    }

    fn render_selected(
        &self,
        canvas: &mut render::Canvas<video::Window>,
        font: &ttf::Font,
        x: i32,
        y: i32,
    ) {
        let text_colour = pixels::Color::RGBA(255, 255, 255, 255);
        let selected_bg_colour = pixels::Color::RGB(50, 200, 20);

        canvas.set_draw_color(selected_bg_colour);
        let _ = canvas.fill_rect(rect::Rect::new(x, y, 200, 50));

        graphics::render_text(canvas, font, text_colour, x, y, &self.text);
    }
}

struct Radio {
    options: Vec<RadioOption>,
    selected_option: usize,
}

impl Radio {
    fn new(options: Vec<RadioOption>) -> Radio {
        Radio {
            options,
            selected_option: 0,
        }
    }

    fn render(&self, canvas: &mut render::Canvas<video::Window>, font: &ttf::Font, x: i32, y: i32) {
        for (idx, option) in self.options.iter().enumerate() {
            if idx == self.selected_option {
                option.render_selected(canvas, font, x + (idx * 200) as i32, y);
            } else {
                option.render(canvas, font, x + (idx * 200) as i32, y);
            }
        }
    }
}

struct PreferencesPage {
    drop_indicator_radio: Radio,
}

impl PreferencesPage {
    fn new() -> PreferencesPage {
        PreferencesPage {
            drop_indicator_radio: Radio::new(vec![
                RadioOption::new("None".to_string()),
                RadioOption::new("Outline".to_string()),
                RadioOption::new("Triangles".to_string()),
            ]),
        }
    }

    fn preferences(&self) -> preferences::Preferences {
        let mut prefs = preferences::Preferences::new();
        prefs.drop_indicator = match self.drop_indicator_radio.selected_option {
            1 => preferences::DropIndicatorStyle::Outline,
            2 => preferences::DropIndicatorStyle::Triangles,
            _ => preferences::DropIndicatorStyle::None,
        };

        prefs
    }

    fn handle_event(&mut self, event: &event::Event) -> bool {
        match event {
            event::Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => match keycode {
                keyboard::Keycode::Left => {
                    if self.drop_indicator_radio.selected_option > 0 {
                        self.drop_indicator_radio.selected_option -= 1
                    }
                }
                keyboard::Keycode::Right => {
                    if self.drop_indicator_radio.selected_option < 2 {
                        self.drop_indicator_radio.selected_option += 1
                    }
                }
                _ => (),
            },
            _ => (),
        }

        false
    }

    fn render(&self, canvas: &mut render::Canvas<video::Window>, font: &ttf::Font) {
        let (canvas_width, canvas_height) = canvas.window().size();
        let canvas_third = canvas_width / 3;
        canvas.set_draw_color(pixels::Color::RGB(200, 80, 13));
        let _ = canvas.fill_rect(rect::Rect::new(
            canvas_third as i32,
            0,
            canvas_third * 2,
            canvas_height,
        ));

        let page_x = canvas_third;

        let c = pixels::Color::RGBA(240, 240, 240, 255);
        graphics::render_text(canvas, font, c, page_x as i32 + 100, 100, "Drop Indicator");

        self.drop_indicator_radio
            .render(canvas, font, page_x as i32 + 100, 150);
    }
}

pub enum MenuOptionSize {
    Regular,
    Large,
}

pub struct MenuOption {
    text: String,
    size: MenuOptionSize,
    selection_action: SelectionAction,
}

impl MenuOption {
    fn new(text: String, size: MenuOptionSize, selection_action: SelectionAction) -> MenuOption {
        MenuOption {
            text,
            size,
            selection_action,
        }
    }
}

pub struct Menu<'ttf, 'rwops> {
    regular_font: ttf::Font<'ttf, 'rwops>,
    large_font: ttf::Font<'ttf, 'rwops>,

    prefs_page: PreferencesPage,
    show_prefs_page: bool,

    options: Vec<MenuOption>,
    selected_option: Option<usize>,
}

impl<'ttf, 'rwops> Menu<'ttf, 'rwops> {
    pub fn new(
        registry: &'rwops assets::Registry,
        ttf_context: &'ttf ttf::Sdl2TtfContext,
    ) -> Result<Menu<'ttf, 'rwops>, String> {
        let font_bytes = registry
            .get("fonts/SourceCodePro-Regular.otf")
            .map_err(|e| e.to_string())?;
        {
            let regular_rwops = rwops::RWops::from_bytes(font_bytes)?;
            let regular_font = ttf_context.load_font_from_rwops(regular_rwops, 28)?;

            let large_rwops = rwops::RWops::from_bytes(font_bytes)?;
            let large_font = ttf_context.load_font_from_rwops(large_rwops, 42)?;

            let mut menu = Menu {
                regular_font,
                large_font,
                prefs_page: PreferencesPage::new(),
                show_prefs_page: false,
                options: vec![],
                selected_option: None,
            };

            menu.options.push(MenuOption::new(
                "Play".to_string(),
                MenuOptionSize::Large,
                SelectionAction::UI(actions::Action::Play),
            ));
            menu.options.push(MenuOption::new(
                "Replays".to_string(),
                MenuOptionSize::Regular,
                SelectionAction::Menu(MenuAction::ShowPreferences),
            ));
            menu.options.push(MenuOption::new(
                "Preferences".to_string(),
                MenuOptionSize::Regular,
                SelectionAction::Menu(MenuAction::ShowPreferences),
            ));

            menu.options.push(MenuOption::new(
                "Quit (q)".to_string(),
                MenuOptionSize::Regular,
                SelectionAction::UI(actions::Action::Quit),
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
            canvas.set_draw_color(pixels::Color::RGBA(222, 222, 222, 255));
            let _ = canvas.fill_rect(rect::Rect::new(50, y - 20, 500, 80));
        }
        let f = match opt.size {
            MenuOptionSize::Regular => &self.regular_font,
            MenuOptionSize::Large => &self.large_font,
        };
        let c = match selected {
            true => pixels::Color::RGBA(50, 180, 50, 255),
            false => pixels::Color::RGBA(255, 255, 255, 255),
        };
        graphics::render_text(canvas, f, c, 100, y, &opt.text);
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        let (canvas_width, canvas_height) = canvas.window().size();
        canvas.set_draw_color(pixels::Color::RGB(200, 100, 13));
        let _ = canvas.fill_rect(rect::Rect::new(0, 0, canvas_width / 3, canvas_height));

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

        if self.show_prefs_page {
            self.prefs_page.render(canvas, &self.regular_font)
        }
    }
    pub fn process_events(&mut self, event_pump: &mut sdl2::EventPump) -> Vec<actions::Action> {
        let mut ui_actions = vec![];
        for event in event_pump.poll_iter() {
            if self.show_prefs_page {
                if self.prefs_page.handle_event(&event) {
                    continue;
                }
            }
            match event {
                event::Event::Quit { .. } => ui_actions.push(actions::Action::Quit),
                event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    keyboard::Keycode::F11 => ui_actions.push(actions::Action::ToggleFullScreen),
                    keyboard::Keycode::Escape => {
                        if self.show_prefs_page {
                            self.show_prefs_page = false;
                            let prefs = self.prefs_page.preferences();
                            ui_actions.push(actions::Action::PreferencesUpdate(prefs));
                        } else {
                            ui_actions.push(actions::Action::MenuHide);
                        }
                    }
                    keyboard::Keycode::Backquote => ui_actions.push(actions::Action::ConsoleShow),
                    keyboard::Keycode::Q => ui_actions.push(actions::Action::Quit),
                    keyboard::Keycode::Down => self.move_down(),
                    keyboard::Keycode::Up => self.move_up(),
                    keyboard::Keycode::Return => self.enter_selection(&mut ui_actions),
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

    fn enter_selection(&mut self, ui_actions: &mut Vec<actions::Action>) {
        if self.selected_option.is_none() {
            return;
        }

        let selected_option = &self.options[self.selected_option.unwrap()];
        match &selected_option.selection_action {
            SelectionAction::UI(action) => ui_actions.push(action.clone()),
            SelectionAction::Menu(action) => match action {
                MenuAction::ShowPreferences => self.show_prefs_page = true,
            },
        }
    }
}
