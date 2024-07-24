use crate::{
    color_util::{self, color, ForegroundBackground, ToAnsiString},
    presets::Preset,
    types,
};
use palette::{
    Alpha, WithAlpha,
    blend::Blend,
    encoding::Srgb,
    rgb::{Rgb, Rgba},
};
use std::{
    io::{self, Write},
    process,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use strum::VariantArray;
use terminal_size::{terminal_size, Height, Width};
use types::AnsiMode;

static KEY_PRESSED: AtomicBool = AtomicBool::new(false);

const TEXT_ASCII: &str = r"
.======================================================.
| .  .              .__       .     .  .       , .   | |
| |__| _.._ ._   .  [__)._.* _| _   |\/| _ ._ -+-|_  | |
| |  |(_][_)[_)\_|  |   [  |(_](/,  |  |(_)[ ) | [ ) * |
|        |  |  ._|                                     |
'======================================================'
";

#[allow(clippy::too_many_lines)]
pub fn start_animation() {
    let mut input: String = String::new();
    //this thread listens for input
    let _listen_thread = thread::spawn(move || {
        match io::stdin().read_line(&mut input) {
            Ok(0) => {},
            Ok(_) => KEY_PRESSED.store(true, Ordering::SeqCst),  // Input detected
            Err(e) => unreachable!("I hope you can't reach this. If so there was an error with reading from stdin: {e}"),  // Error reading input
        }
    });

    // the "happy pride month" text that is displayed
    let text = &TEXT_ASCII[1..TEXT_ASCII.len() - 1];

    let text_lines: Vec<&str> = text.split('\n').collect();
    let text_height: usize = text_lines.len();
    let text_width: usize = text_lines[0].len();

    let notice = "Press any key to continue";

    let speed = 2;
    let frame_delay: Duration = Duration::from_secs_f32(1.0 / 25.0);

    // colors: list[RGB] = []
    let mut frame: usize = 0;

    let size = terminal_size();

    let (term_width, term_height) = if let Some((Width(w), Height(h))) = size {
        (w as usize, h as usize)
    } else {
        panic!("Could not resolve terminal size");
    };

    //all the variables needed for the animation
    let blocks: usize = 9;
    let block_width: usize = term_width / blocks;

    let text_start_y = (term_height / 2) - (text_height / 2);
    let text_end_y = text_start_y + text_height;
    let text_start_x = (term_width / 2) - (text_width / 2);
    let text_end_x = text_start_x + text_width;

    let notice_start_x = term_width - notice.len() - 1;
    let notice_end_x = term_width - 1;
    let notice_y = term_height - 1;
    let colors: Vec<Rgb<Srgb, u8>> = Preset::VARIANTS
        .iter()
        .flat_map(|p| p.color_profile().colors)
        .collect::<Vec<_>>();

    //used for foreground and black color overlay, respectively
    let fg = "#FFE09B".parse::<Rgb<Srgb, u8>>().unwrap();
    let black: Rgba<Srgb, f64> = Rgba::<Srgb, f64>::new(0.0, 0.0, 0.0, 0.5);

    loop {
        //clear screen first
        color_util::clear_screen(None, AnsiMode::Rgb, false).unwrap();

        {
            //this buffer holds all of the color
            let mut buf: String = String::new();

            for y in 0..term_height {
                buf += &colors[((frame + y) / block_width) % colors.len()]
                    .to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Background);
                buf += &fg.to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Foreground);

                let mut x = 0;
                //loop over width
                while x < term_width {
                    let idx =
                        frame + x + y + (2.0 * (y as f64 + 0.5 * frame as f64).sin()) as usize;
                    let y_text = text_start_y <= y && y < text_end_y;
                    let border = 1 + usize::from(!(y == text_start_y || y == text_end_y - 1));
                    //if switching point
                    if idx % block_width == 0
                        || x == text_start_x - border
                        || x == text_end_x + border
                        || x == notice_start_x - 1
                        || x == notice_end_x + 1
                    {
                        //print the color of the current frame

                        let c: Alpha<Rgb<Srgb, f64>, f64> = colors
                            [(idx / block_width) % colors.len()]
                        .into_format::<f64>()
                        .into();

                        if (y_text && (text_start_x - border <= x) && (x < text_end_x + border))
                            || (y == notice_y && notice_start_x - 1 <= x && x < notice_end_x + 1)
                        {
                            buf += &c
                                .overlay(black) //make the background darker
                                .without_alpha()   //remove alpha wrapper
                                .into_format::<u8>()
                                .to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Background);
                        } else {
                            buf += &c
                                .without_alpha()
                                .into_format()
                                .to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Background);
                        }
                    }

                    if y_text && text_start_x <= x && x < text_end_x {
                        buf.push(
                            text_lines[y - text_start_y]
                                .chars()
                                .nth(x - text_start_x)
                                .unwrap(),
                        );
                    } else if y == notice_y && notice_start_x <= x && x < notice_end_x {
                        buf.push(notice.chars().nth(x - notice_start_x).unwrap());
                    } else {
                        buf.push(' ');
                    }

                    x += 1;
                }

                if y != term_height - 1 {
                    buf += &color("&r\n", AnsiMode::Rgb).unwrap();
                }
            }
            //flush stdout
            print!("{buf}");

            io::stdout().flush().unwrap();
        };
        frame += speed;
        thread::sleep(frame_delay);
        if KEY_PRESSED.load(Ordering::SeqCst) {
            //this ansi code is needed in that it resets the background.
            //otherwise we get a lot of nasty background color left over!
            //the color_util::clear_screen should implement it in the future
            //so that it isn't needed here.
            print!("\x1b[49m");
            io::stdout().flush().unwrap();
            color_util::clear_screen(None, AnsiMode::Rgb, false).unwrap();
            process::exit(0);
        }
    }
}
