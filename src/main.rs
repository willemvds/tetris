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

extern crate sdl2;
use sdl2::pixels;

#[rustfmt::skip]
const ASSET_MANIFEST: [&str; 2] = [
    "fonts/PressStart2P-Regular.ttf",
    "fonts/SourceCodePro-Regular.otf"
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

fn main() -> Result<(), String> {
    let mut ui_layers = UILayers::new();
    let prefs = preferences::Preferences::new();

    let mut registry = assets::Registry::new();
    for asset in ASSET_MANIFEST.iter() {
        let content = fs::read(format!("assets/{}", asset)).map_err(|e| e.to_string())?;
        registry.insert(asset, content)
    }

    let args: Vec<String> = env::args().collect();

    let mut replay: Option<replays::Replay> = None;
    let mut last_game = None;
    if args.len() > 1 {
        let recording_file = fs::File::open(args[1].clone()).map_err(|e| e.to_string())?;
        let recording_file_reader = io::BufReader::new(recording_file);
        let recording =
            serde_json::from_reader(recording_file_reader).map_err(|e| e.to_string())?;
        replay = Some(replays::Replay { recording });
    } else {
        match load_last_game_state() {
            Ok(lgs) => last_game = Some(lgs),
            Err(e) => println!("{}", e),
        }
    }

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut console = console::Console::new(&registry, &ttf_context)?;
    let mut menu = menu::Menu::new(&registry, &ttf_context)?;

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
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    let _total = 0;
    let mut frames = 0;
    let mut slowest_frame = 0.0;

    let dt: f64 = 1.0 / 250.0; // 4ms tick rate.

    let game_loop_start_at = time::Instant::now();
    let mut start_time = time::Instant::now();

    let mut game_rules = game::Rules::new();
    game_rules.lock_delay(50);
    // game_rules.lock_delay_on_hard_drop(true);

    let mut game_shell = match replay {
        Some(rp) => {
            let replay_pieces = replays::ReplayPieces::new(&rp);
            let gm = game::Game::new(game_rules.clone(), Some(Box::new(replay_pieces)))?;
            game_shell::GameShell::new_with_replay(gm, rp)
        }
        None => {
            if let Some(g) = last_game {
                game_shell::GameShell::new(g)
            } else {
                game_shell::GameShell::new(game::Game::new(game_rules.clone(), None)?)
            }
        }
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
                actions::Action::NewGame => {
                    let _ = game_shell.new_game(game_rules.clone());
                }
                actions::Action::QueueGameAction(a) => {
                    let _ = game_shell.queue_action(*a);
                }
                actions::Action::TogglePause => game_shell.toggle_pause(),
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
                    if cmd == "quit" {
                        break 'main;
                    }
                    if cmd == "speed" {
                        //                        console.println(format!("Speed = {0}", game.speed));
                    } else {
                        console.println("EH wha?".to_string());
                    }
                    println!("CONSOLE CMD = {0}", cmd);
                }
            }
        }

        game_shell.frame_tick(frame_time, dt);

        game_shell.render(&mut canvas, &prefs, &font);

        graphics::render_text(
            &mut canvas,
            &font,
            pixels::Color::RGB(0, 0, 255),
            20,
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
