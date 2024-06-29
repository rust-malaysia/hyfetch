use hyfetch::presets::Preset;
use rgb::RGB8;
use ansi_rgb::{Foreground};
use hyfetch::color_util::FromHex;
use term_size::{dimensions};
use std::io::{self, Write};
use clearscreen::clear;

fn  strip_trailing_nl(input: &mut String) {
    let strip_len = input
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, '\n' | '\r'))
        .map_or(0, |(i, _)| i + 1);
    if strip_len != input.len() {
        input.truncate(strip_len);
    }
}

fn  start_animation() {
    let mut text =  r#".======================================================.
| .  .              .__       .     .  .       , .   | |
| |__| _.._ ._   .  [__)._.* _| _   |\/| _ ._ -+-|_  | |
| |  |(_][_)[_)\_|  |   [  |(_](/,  |  |(_)[ ) | [ ) * |
|        |  |  ._|                                     |
'======================================================'"#.to_string();

    strip_trailing_nl(&mut text);
    let text_lines: Vec<&str> = text.split("\n").collect();
    let text_height = text_lines.len() as f64;
    let text_width = text_lines[0].len() as f64;

    let notice = "Press enter to continue";

    let speed = 2;
    let frame_delay = 1 / 25;

    // colors: list[RGB] = []
    let mut colors: Vec<RGB8> = Vec::new();
    let frame = 0;

    fn term_size() -> Option<(usize, usize)> {
        dimensions()}
    let mut w: f64 = 0.0;
    let mut h: f64 = 0.0;
    if let Some((width, height)) = term_size() {
        w = width as f64;
        h = height as f64;
        println!("Terminal size: {} x {}", w, h);
    }
    else {
        println!("Couldn't get terminal size");
    }

    let blocks: f64 = 9.0;
    let block_width: f64 = (w / blocks).floor();

    let text_start_y: f64  = (h / 2.0).floor() - (text_height / 2.0).floor();
    let text_end_y: f64 = text_start_y + text_height;
    let text_start_x: f64  = (w / 2.0).floor() - (text_width / 2.0).floor();
    let text_end_x: f64  = text_start_x + text_width;

    let notice_start_x: f64  = w - notice.len() as f64 - 1.0;
    let notice_end_x: f64 = w - 1.0;
    let notice_y: f64 = h - 1.0;

    //# Add everything in PRESETS to colors
    //colors = [c for preset in PRESETS.values() for c in preset.colors]
    let colors: Vec<RGB8> = Preset::color_profile.into_iter().map(|p| p.color_profile()).collect();

    let black = RGB8 {r:0, g:0, b:0};
    let fg = RGB8::from_hex("FFE098");

    let non_dynamic_h = h;
    //fn draw_frame() {
    let draw_frame = || {
        let buf = "";

        //for y in 0..non_dynamic_h {
        for y in 0..100 {
            //buf += colors[((frame + y) // block_width) % len(colors)].to_ansi_rgb(foreground=False)
            //buf += fg.to_ansi_rgb(foreground=True)
            buf += colors[(((frame as f64 + y as f64) / block_width).floor()) % colors.len()];
            // TODO - error
            //buf += fg(fg);

            let mut x = 0;
            while x < w as i32 {
                let idx = frame + x + y + (f64::sin((y as f64) + 0.5 * (frame as f64)) * 2.0) as i32;
                let y_text = text_start_y <= y as f64 && (y as f64) < text_end_y;
                let border = 1 + (!(y as f64 == text_start_y || y as f64 == text_end_y - 1 as f64)) as i32;

                if idx % block_width as i32 == 0 || x as f64 == text_start_x - border as f64|| x as f64 == text_end_x + border as f64 || x as f64 == notice_start_x - 1 as f64 || x as f64 == notice_end_x + 1 as f64
                {
                    let c = colors[((idx as f64 / block_width).floor()) % colors.len() as f64];
                    if ((y_text && text_start_x - border as f64 <= x as f64) && ((x as f64) < text_end_x + border as f64)) || ((y as f64 == notice_y && notice_start_x - (1 as f64) < x as f64) && ((x as f64) < notice_end_x + 1 as f64)) {
                        //buf += c.overlay(black, 0.5).to_ansi_rgb(foreground=False)
                        buf += c;
                    }
                    else {
                        buf += c
                    }
                // TODO - error
                // if y_text && text_start_x <= x as f64 && (x as f64) < text_end_x {
                    // buf += text_lines[y as f64 - text_start_y][x as f64 - text_start_x];
                }
                // TODO - error
                // else if y as f64 == notice_y && notice_start_x <= x as f64 && (x as f64) < notice_end_x {
                    // buf += notice[x as f64 - notice_start_x];
                }
                else {
                    // TODO - error
                    // buf += ' ';
                }

                x += 1;
                }

            if y as f64 != h - 1 as f64 {
                //buf += color('&r\n')
            }
            }
        }

        print!("{}", buf);
        io::stdout().flush().unwrap();
    };

    // catch exception

    // clear screen
}

fn  main() {
    start_animation();
}
