use crate::tetris::tetrominos;

use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::ttf;
use sdl2::video;

pub fn render_text(
    canvas: &mut render::Canvas<video::Window>,
    font: &ttf::Font,
    colour: pixels::Color,
    x: i32,
    y: i32,
    text: &str,
) {
    let texture_creator = canvas.texture_creator();

    let (char_width, char_height) = font.size_of_char('C').unwrap();

    let surface = font
        .render(text)
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

pub fn render_text_centered(
    canvas: &mut render::Canvas<video::Window>,
    font: &ttf::Font,
    colour: pixels::Color,
    x: i32,
    y: i32,
    text: &str,
) {
    let texture_creator = canvas.texture_creator();

    let (char_width, char_height) = font.size_of_char('C').unwrap();

    let surface = font
        .render(text)
        .blended(colour)
        .map_err(|e| e.to_string())
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())
        .unwrap();

    let text_width = char_width * text.len() as u32;
    let target_x = x - (text_width / 2) as i32;
    let target_y = y - (char_height / 2) as i32;

    let target = rect::Rect::new(target_x, target_y, text_width, char_height);

    let _ = canvas.copy(&texture, None, Some(target));
}

pub fn render_form(
    canvas: &mut render::Canvas<video::Window>,
    s: tetrominos::Form,
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

pub fn tetromino_colour(kind: tetrominos::Kind) -> pixels::Color {
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
