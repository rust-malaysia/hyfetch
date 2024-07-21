//by 1337isnot1337

use crate::{
    color_util::{color, ForegroundBackground, ToAnsiString},
    presets::Preset,
    types,
};
use palette::{encoding::Srgb, rgb::Rgb};
use std::{
    io::{self, Write},
    process,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use types::AnsiMode;

use term_size::dimensions;

static KEY_PRESSED: AtomicBool = AtomicBool::new(false);

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
    let text = r".======================================================.
| .  .              .__       .     .  .       , .   | |
| |__| _.._ ._   .  [__)._.* _| _   |\/| _ ._ -+-|_  | |
| |  |(_][_)[_)\_|  |   [  |(_](/,  |  |(_)[ ) | [ ) * |
|        |  |  ._|                                     |
'======================================================'";

    let text_lines: Vec<&str> = text.split('\n').collect();
    let text_height: usize = text_lines.len();
    let text_width: usize = text_lines[0].len();

    let notice = "Press any key to continue";

    let speed = 2;
    let frame_delay: Duration = Duration::from_secs_f32(1.0 / 25.0);

    // colors: list[RGB] = []
    let mut frame: usize = 0;

    let Some((w, h)) = dimensions() else {
        panic!("Couldn't get terminal size");
    };
    //all the variables needed for the animation
    let blocks: usize = 9;
    let block_width: usize = w / blocks;

    let text_start_y = (h / 2) - (text_height / 2);
    let text_end_y = text_start_y + text_height;
    let text_start_x = (w / 2) - (text_width / 2);
    let text_end_x = text_start_x + text_width;

    let notice_start_x = w - notice.len() - 1;
    let notice_end_x = w - 1;
    let notice_y = h - 1;

    //it is inefficient but I don't know how to iter over values in an enum
    let colors = [
        Preset::Rainbow,
        Preset::Transgender,
        Preset::Nonbinary,
        Preset::Xenogender,
        Preset::Agender,
        Preset::Queer,
        Preset::Genderfluid,
        Preset::Bisexual,
        Preset::Pansexual,
        Preset::Polysexual,
        Preset::Omnisexual,
        Preset::Omniromantic,
        Preset::GayMen,
        Preset::Lesbian,
        Preset::Abrosexual,
        Preset::Asexual,
        Preset::Aromantic,
        Preset::Aroace1,
        Preset::Aroace2,
        Preset::Aroace3,
        Preset::Greysexual,
        Preset::Autosexual,
        Preset::Intergender,
        Preset::Greygender,
        Preset::Akiosexual,
        Preset::Bigender,
        Preset::Demigender,
        Preset::Demiboy,
        Preset::Demigirl,
        Preset::Transmasculine,
        Preset::Transfeminine,
        Preset::Genderfaun,
        Preset::Demifaun,
        Preset::Genderfae,
        Preset::Demifae,
        Preset::Neutrois,
        Preset::Biromantic1,
        Preset::Autoromantic,
        Preset::Boyflux2,
        Preset::Girlflux,
        Preset::Genderflux,
        Preset::Finsexual,
        Preset::Unlabeled1,
        Preset::Unlabeled2,
        Preset::Pangender,
        Preset::GenderNonconforming1,
        Preset::GenderNonconforming2,
        Preset::Femboy,
        Preset::Tomboy,
        Preset::Gynesexual,
        Preset::Androsexual,
        Preset::Gendervoid,
        Preset::Voidgirl,
        Preset::Voidboy,
        Preset::NonhumanUnity,
        Preset::Plural,
        Preset::Fraysexual,
        // Meme flag
        Preset::Beiyang,
        // Meme flag
        Preset::Burger,
        Preset::Baker,
    ]
    .into_iter()
    .flat_map(|p| p.color_profile().colors)
    .collect::<Vec<_>>();
    //need to figure out how to print a darker background for the text, will use black for it
    //once i figure out how to do it
    /*let black: Rgb<Srgb, u8> = Rgb {
        red: 0,
        green: 0,
        blue: 0,
        standard: PhantomData,
    };*/
    let fg = "#FFE09B".parse::<Rgb<Srgb, u8>>().unwrap();

    loop {
        //clear screen first
        match clearscreen::clear() {
            Ok(()) => {},
            Err(e) => panic!("Failed to clear screen: {e}"),
        }

        {
            //this buffer holds all of the color
            let mut buf: String = String::new();

            for y in 0..h {
                buf += &colors[((frame + y) / block_width) % colors.len()]
                    .to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Background);
                buf += &fg.to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Foreground);

                let mut x = 0;
                //loop over width
                while x < w {
                    let idx =
                    //potential truncation worry but it should be fine
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
                        let c = colors[(idx / block_width) % colors.len()];
                        if (y_text && (text_start_x - border <= x) && (x < text_end_x + border))
                            || (y == notice_y && notice_start_x - 1 <= x && x < notice_end_x + 1)
                        {
                            //buf += c.overlay(black, 0.5).to_ansi_rgb(foreground=False)
                            buf += // <--- && ^^^^^^ need to print a darker background somehow
                                &c.to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Background);
                        } else {
                            buf +=
                                &c.to_ansi_string(AnsiMode::Rgb, ForegroundBackground::Background);
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

                if y != h - 1 {
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
            print!("\x1b[2J\x1b[H");
            io::stdout().flush().unwrap();
            clearscreen::clear().unwrap();

            process::exit(0);
        }
    }
}

