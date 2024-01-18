`

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

