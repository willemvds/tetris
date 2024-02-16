use std::collections;
use std::env;
use std::fs;
use std::io;
use std::time;

mod actions;
mod assets;
mod console;
mod game_shell;
mod graphics;
mod menu;
mod preferences;
mod replays;
mod tetris;
use tetris::game;
use tetris::tetrominos;

extern crate sdl2;
use sdl2::controller;
use sdl2::keyboard;
use sdl2::pixels;
use sdl2::video;

#[rustfmt::skip]
const ASSET_MANIFEST: [&str; 2] = [
//    "fonts/NotoSansMono-Regular.ttf",
    "fonts/PressStart2P-Regular.ttf",
    "fonts/SourceCodePro-Regular.otf",
];

const UI_LAYER_GAME: u8 = 0b0001;
const UI_LAYER_CONSOLE: u8 = 0b0000_0100;
const UI_LAYER_MENU: u8 = 0b0000_0010;
const UI_LAYER_CINEMA: u8 = 0b0000_1000;
const UI_LAYER_OVERLAY: u8 = 0b0001_0000;

struct UILayers {
    layers: u8,
}

impl UILayers {
    fn new() -> UILayers {
        UILayers {
            layers: UI_LAYER_GAME | UI_LAYER_MENU,
        }
    }

    fn hide(&mut self, layer: u8) {
        self.layers ^= layer
    }

    fn show(&mut self, layer: u8) {
        self.layers |= layer
    }

    fn is_showing(&self, layer: u8) -> bool {
        self.layers & layer == layer
    }
}

fn load_preferences_from_file(path: &str) -> Result<preferences::Preferences, String> {
    if let Ok(preferences_file) = fs::File::open(path) {
        let preferences_reader = io::BufReader::new(preferences_file);
        let prefs: preferences::Preferences =
            serde_json::from_reader(preferences_reader).map_err(|e| e.to_string())?;
        return Ok(prefs);
    }

    Err("Preferences not found".to_string())
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

    Err("Previous game state not available.".to_string())
}

fn load_replay(path: &str) -> Result<replays::Replay, String> {
    let recording_file = fs::File::open(path).map_err(|e| e.to_string())?;
    let recording_file_reader = io::BufReader::new(recording_file);
    let recording = serde_json::from_reader(recording_file_reader).map_err(|e| e.to_string())?;

    Ok(replays::Replay { recording })
}

fn main() -> Result<(), String> {
    let mut ui_layers = UILayers::new();
    let mut prefs = preferences::Preferences::new();

    let mut registry = assets::Registry::new();
    for asset in ASSET_MANIFEST.iter() {
        let content = fs::read(format!("assets/{}", asset)).map_err(|e| e.to_string())?;
        registry.insert(asset, content)
    }

    match load_preferences_from_file("preferences.json") {
        Ok(preferences) => prefs = preferences,
        Err(err) => {
            println!("err = {}", err)
        }
    }

    let args: Vec<String> = env::args().collect();

    let mut replay: Option<replays::Replay> = None;
    let mut last_game = None;
    if args.len() > 1 {
        replay = match load_replay(&args[1]) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    } else {
        match load_last_game_state() {
            Ok(lgs) => last_game = Some(lgs),
            Err(e) => println!("{}", e),
        }
    }

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let game_controller_subsys = sdl_context.game_controller()?;

    let num_joysticks = game_controller_subsys.num_joysticks()?;

    let mut game_controllers: Vec<controller::GameController> = vec![];
    for id in 0..num_joysticks {
        if !game_controller_subsys.is_game_controller(id) {
            continue;
        }

        if let Ok(controller) = game_controller_subsys.open(id) {
            println!(
                "Game Controller ({}) ATTACHED = {}",
                controller.name(),
                controller.mapping()
            );
            game_controllers.push(controller);
        }
    }

    let mut console = console::Console::new(&registry, &ttf_context)?;

    let mut replay_paths = vec![];
    if let Ok(dir_iter) = fs::read_dir("replays") {
        for entry in dir_iter {
            if let Ok(file) = entry {
                let path = file.path();
                if path.is_dir() {
                    continue;
                }
                if let Some(path_str) = path.to_str() {
                    replay_paths.push(path_str.to_string())
                }
            }
        }
    }
    let mut menu = menu::Menu::new(&registry, &ttf_context, prefs.clone(), replay_paths)?;

    let mut font = ttf_context.load_font_from_rwops(
        registry
            .get_rwops("fonts/SourceCodePro-Regular.otf")
            .map_err(|e| e.to_string())?,
        48,
    )?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

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
        .borderless()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.present();

    let keymap: collections::HashMap<keyboard::Keycode, u8> = collections::HashMap::new();

    let mut event_pump = sdl_context.event_pump()?;

    let _total = 0;
    let mut frames = 0;
    let mut slowest_frame = 0.0;

    let dt: f64 = 1.0 / 250.0; // 4ms tick rate.

    let game_loop_start_at = time::Instant::now();
    let mut start_time = time::Instant::now();

    let mut game_rules = tetris::rules::Rules::new();
    game_rules.lock_delay(50);
    // game_rules.lock_delay_on_hard_drop(true);

    let mut game_shell = game_shell::GameShell::new(
        game::Game::new(game_rules.clone(), None)?,
        &registry,
        &ttf_context,
    )?;

    if let Some(rp) = replay {
        let replay_pieces = replays::ReplayPieces::new(&rp);
        let replay_game = game::Game::new(game_rules.clone(), Some(Box::new(replay_pieces)))?;
        game_shell.load_replay(replay_game, rp)
    } else if let Some(lg) = last_game {
        game_shell.load_game(lg)
    };

    if game_shell.is_showing_replay() {
        game_shell.unpause();
        ui_layers.hide(UI_LAYER_MENU);
    }

    'main: loop {
        frames += 1;
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

        let ui_actions = {
            if ui_layers.is_showing(UI_LAYER_CONSOLE) {
                console.process_events(&mut event_pump)
            } else if ui_layers.is_showing(UI_LAYER_MENU) {
                menu.process_events(&mut event_pump)
            } else {
                game_shell.process_events(&mut event_pump)
            }
        };

        for action in ui_actions.iter() {
            match action {
                actions::Action::Quit => break 'main,
                actions::Action::PreferencesUpdate(p) => prefs = p.clone(),
                actions::Action::Play => {
                    if game_shell.is_gameover() {
                        let new_game = game::Game::new(game_rules.clone(), None)?;
                        game_shell.load_game(new_game);
                    }
                    ui_layers.hide(UI_LAYER_MENU);
                    game_shell.unpause();
                }
                actions::Action::NewGame => {
                    let new_game = game::Game::new(game_rules.clone(), None)?;
                    game_shell.load_game(new_game);
                }
                actions::Action::ReplayLoad(path) => match load_replay(path) {
                    Ok(r) => {
                        let replay_pieces = replays::ReplayPieces::new(&r);
                        let replay_game =
                            game::Game::new(game_rules.clone(), Some(Box::new(replay_pieces)))?;
                        game_shell.load_replay(replay_game, r)
                    }
                    Err(_) => (),
                },
                actions::Action::TogglePause => game_shell.toggle_pause(),
                actions::Action::ToggleFullScreen => {
                    if canvas.window().fullscreen_state() == video::FullscreenType::Off {
                        let _ = canvas
                            .window_mut()
                            .set_fullscreen(video::FullscreenType::Desktop);
                    } else {
                        let _ = canvas
                            .window_mut()
                            .set_fullscreen(video::FullscreenType::Off);
                    }
                }
                actions::Action::MenuShow => {
                    ui_layers.show(UI_LAYER_MENU);
                    game_shell.pause();
                }
                actions::Action::MenuHide => {
                    ui_layers.hide(UI_LAYER_MENU);
                    game_shell.unpause();
                }
                actions::Action::ConsoleShow => ui_layers.show(UI_LAYER_CONSOLE),
                actions::Action::ConsoleHide => ui_layers.hide(UI_LAYER_CONSOLE),
                actions::Action::ConsoleCommand(cmd) => {
                    match cmd.as_str() {
                        "quit" => break 'main,
                        "speed" => (),
                        "stick" => console.print_tetromino(tetrominos::Kind::Stick),
                        "seven" => console.print_tetromino(tetrominos::Kind::Seven),
                        "hook" => console.print_tetromino(tetrominos::Kind::Hook),
                        "square" => console.print_tetromino(tetrominos::Kind::Square),
                        "snake" => console.print_tetromino(tetrominos::Kind::Snake),
                        "pyramid" => console.print_tetromino(tetrominos::Kind::Pyramid),
                        "zig" => console.print_tetromino(tetrominos::Kind::Zig),
                        _ => console.println("EH wha?".to_string()),
                    }
                    println!("CONSOLE CMD = {0}", cmd);
                }
            }
        }

        game_shell.frame_tick(frame_time, dt);

        game_shell.render(&mut canvas, &prefs);

        let (ww, _) = canvas.window().size();
        graphics::render_text(
            &mut canvas,
            &font,
            pixels::Color::RGB(0, 0, 255),
            ww as i32 - 400,
            20,
            &format!("{:.2} fps", frame_rate),
        );

        if ui_layers.is_showing(UI_LAYER_MENU) {
            menu.render(&mut canvas);
        }

        if ui_layers.is_showing(UI_LAYER_CONSOLE) {
            console.render(&mut canvas);
        }

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

    let mut preferences_file = fs::File::create("preferences.json").map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut preferences_file, &prefs).map_err(|e| e.to_string())?;

    if let Ok(recording) = game_shell.recording() {
        let mut recording_file =
            fs::File::create("last_game_recording.json").map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut recording_file, recording).map_err(|e| e.to_string())?;
    }

    if !game_shell.is_showing_replay() {
        let mut last_game_state_file =
            fs::File::create("last_game_state.json").map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut last_game_state_file, game_shell.game())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
