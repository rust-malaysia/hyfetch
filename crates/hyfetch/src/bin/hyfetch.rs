use std::borrow::Cow;
use std::cmp;
use std::fs::{self, File};
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use deranged::RangedU8;
use hyfetch::cli_options::options;
use hyfetch::color_util::{clear_screen, color, printc, ForegroundBackground, Lightness, Theme};
use hyfetch::models::Config;
#[cfg(windows)]
use hyfetch::neofetch_util::ensure_git_bash;
use hyfetch::neofetch_util::{self, ascii_size, get_distro_ascii, ColorAlignment};
use hyfetch::presets::{AssignLightness, Preset};
use hyfetch::types::{AnsiMode, Backend, TerminalTheme};
use hyfetch::utils::get_cache_path;
use palette::Srgb;
use strum::{EnumCount, VariantArray, VariantNames};
use terminal_colorsaurus::{background_color, QueryOptions};
use terminal_size::terminal_size;
use time::{Month, OffsetDateTime};
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

const TEST_ASCII: &str = r####################"
### |\___/| ###
### )     ( ###
## =\     /= ##
#### )===( ####
### /     \ ###
### |     | ###
## / {txt} \ ##
## \       / ##
_/\_\_   _/_/\_
|##|  ( (  |##|
|##|   ) ) |##|
|##|  (_(  |##|
"####################;

fn main() -> Result<()> {
    #[cfg(windows)]
    if let Err(err) = enable_ansi_support::enable_ansi_support() {
        debug!(%err, "could not enable ANSI escape code support");
    }

    let options = options().run();

    let debug_mode = options.debug;

    init_tracing_subsriber(debug_mode).context("failed to init tracing subscriber")?;

    debug!(?options, "CLI options");

    // Use a custom distro
    let distro = options.distro.as_ref();

    let backend = options.backend.unwrap_or(Backend::Neofetch);
    let use_overlay = options.overlay;

    #[cfg(windows)]
    ensure_git_bash().context("failed to find git bash")?;

    if options.test_print {
        let (asc, _) = get_distro_ascii(distro, backend).context("failed to get distro ascii")?;
        println!("{asc}");
        return Ok(());
    }

    let config = if options.config {
        create_config(
            &options.config_file,
            distro,
            backend,
            use_overlay,
            debug_mode,
        )
        .context("failed to create config")?
    } else if let Some(config) =
        read_config(&options.config_file).context("failed to read config")?
    {
        config
    } else {
        create_config(
            &options.config_file,
            distro,
            backend,
            use_overlay,
            debug_mode,
        )
        .context("failed to create config")?
    };

    let color_mode = options.mode.unwrap_or(config.mode);
    let theme = config.light_dark;

    // Check if it's June (pride month)
    let now =
        OffsetDateTime::now_local().context("failed to get current datetime in local timezone")?;
    let cache_path = get_cache_path().context("failed to get cache path")?;
    let june_path = cache_path.join(format!("animation-displayed-{year}", year = now.year()));
    let show_pride_month = options.june
        || now.month() == Month::June && !june_path.is_file() && io::stdout().is_terminal();

    if show_pride_month && !config.pride_month_disable {
        // TODO
        // pride_month.start_animation();
        println!();
        println!("Happy pride month!");
        println!("(You can always view the animation again with `hyfetch --june`)");
        println!();

        if !june_path.is_file() {
            fs::create_dir_all(&cache_path)
                .with_context(|| format!("failed to create cache dir {cache_path:?}"))?;
            File::create(&june_path)
                .with_context(|| format!("failed to create file {june_path:?}"))?;
        }
    }

    // Use a custom distro
    let distro = options.distro.as_ref().or(config.distro.as_ref());

    let backend = options.backend.unwrap_or(config.backend);
    let args = options.args.as_ref().or(config.args.as_ref());

    // Get preset
    let preset = options.preset.unwrap_or(config.preset);
    let color_profile = preset.color_profile();
    debug!(?color_profile, "color profile");

    // Lighten
    let color_profile = if let Some(scale) = options.scale {
        color_profile.lighten(scale)
    } else if let Some(lightness) = options.lightness {
        color_profile.with_lightness(AssignLightness::Replace(lightness))
    } else {
        color_profile.with_lightness_adaptive(config.lightness(), theme, use_overlay)
    };
    debug!(?color_profile, "lightened color profile");

    let (asc, fore_back) = if let Some(path) = options.ascii_file {
        (
            fs::read_to_string(&path)
                .with_context(|| format!("failed to read ascii from {path:?}"))?,
            None,
        )
    } else {
        get_distro_ascii(distro, backend).context("failed to get distro ascii")?
    };
    let color_align = if fore_back.is_some() {
        match config.color_align {
            ca @ ColorAlignment::Horizontal { .. } | ca @ ColorAlignment::Vertical { .. } => {
                ca.with_fore_back(fore_back).context(
                    "failed to create color alignment with foreground-background configuration",
                )?
            },
            ca @ ColorAlignment::Custom { .. } => ca,
        }
    } else {
        config.color_align
    };
    let asc = color_align
        .recolor_ascii(asc, &color_profile, color_mode, theme)
        .context("failed to recolor ascii")?;
    neofetch_util::run(asc, backend, args)?;

    if options.ask_exit {
        print!("Press any key to exit...");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .context("failed to read line from input")?;
    }

    Ok(())
}

/// Reads config from file.
///
/// Returns `None` if the config file does not exist.
#[tracing::instrument(level = "debug")]
fn read_config(path: &Path) -> Result<Option<Config>> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        },
        Err(err) => {
            return Err(err).with_context(|| format!("failed to open {path:?}"));
        },
    };

    let mut buf = String::new();

    file.read_to_string(&mut buf)
        .with_context(|| format!("failed to read {path:?}"))?;

    let deserializer = &mut serde_json::Deserializer::from_str(&buf);
    let config: Config = serde_path_to_error::deserialize(deserializer)
        .with_context(|| format!("failed to parse {path:?}"))?;

    debug!(?config, "read config");

    Ok(Some(config))
}

/// Creates config interactively.
///
/// The config is automatically stored to file.
#[tracing::instrument(level = "debug")]
fn create_config(
    path: &Path,
    distro: Option<&String>,
    backend: Backend,
    use_overlay: bool,
    debug_mode: bool,
) -> Result<Config> {
    // Detect terminal environment (doesn't work for all terminal emulators,
    // especially on Windows)
    let det_bg = if io::stdout().is_terminal() {
        match background_color(QueryOptions::default()) {
            Ok(bg) => Some(Srgb::<u16>::new(bg.r, bg.g, bg.b).into_format::<u8>()),
            Err(terminal_colorsaurus::Error::UnsupportedTerminal) => None,
            Err(err) => {
                return Err(err).context("failed to get terminal background color");
            },
        }
    } else {
        None
    };
    debug!(?det_bg, "detected background color");
    let det_ansi = supports_color::on(supports_color::Stream::Stdout).map(|color_level| {
        if color_level.has_16m {
            AnsiMode::Rgb
        } else if color_level.has_256 {
            AnsiMode::Ansi256
        } else if color_level.has_basic {
            AnsiMode::Ansi16
        } else {
            unreachable!();
        }
    });
    debug!(?det_ansi, "detected color mode");

    let (asc, fore_back) =
        get_distro_ascii(distro, backend).context("failed to get distro ascii")?;
    let (asc_width, asc_lines) = ascii_size(asc);
    let theme = det_bg.map(|bg| bg.theme()).unwrap_or(TerminalTheme::Light);
    let color_mode = det_ansi.unwrap_or(AnsiMode::Ansi256);
    let logo = color(
        match theme {
            TerminalTheme::Light => "&l&bhyfetch&~&L",
            TerminalTheme::Dark => "&l&bhy&ffetch&~&L",
        },
        color_mode,
    )
    .expect("logo should not contain invalid color codes");
    let mut title = format!("Welcome to {logo} Let's set up some colors first.");
    clear_screen(Some(&title), color_mode, debug_mode)
        .expect("title should not contain invalid color codes");

    let mut option_counter: u8 = 1;

    fn update_title(title: &mut String, option_counter: &mut u8, k: &str, v: &str) {
        let k: Cow<str> = if k.ends_with(':') {
            k.into()
        } else {
            format!("{k}:").into()
        };
        title.push_str(&format!("\n&e{option_counter}. {k:<30} &~{v}"));
        *option_counter += 1;
    }

    fn print_title_prompt(option_counter: u8, prompt: &str, color_mode: AnsiMode) {
        printc(format!("&a{option_counter}. {prompt}"), color_mode)
            .expect("prompt should not contain invalid color codes");
    }

    //////////////////////////////
    // 0. Check term size

    // TODO

    //////////////////////////////
    // 1. Select color mode

    let select_color_mode = || -> Result<(AnsiMode, &str)> {
        if det_ansi == Some(AnsiMode::Rgb) {
            return Ok((AnsiMode::Rgb, "Detected color mode"));
        }

        clear_screen(Some(&title), color_mode, debug_mode)
            .expect("title should not contain invalid color codes");

        // TODO

        println!();
        print_title_prompt(
            option_counter,
            "Which &bcolor system &ado you want to use?",
            color_mode,
        );
        printc(
            r#"(If you can't see colors under "RGB Color Testing", please choose 8bit)"#,
            color_mode,
        )
        .expect("message should not contain invalid color codes");
        println!();

        todo!()
    };

    let color_mode = {
        let (color_mode, ttl) = select_color_mode().context("failed to select color mode")?;
        debug!(?color_mode, "selected color mode");
        update_title(&mut title, &mut option_counter, ttl, color_mode.into());
        color_mode
    };

    //////////////////////////////
    // 2. Select theme (light/dark mode)

    let select_theme = || -> Result<(TerminalTheme, &str)> {
        if let Some(det_bg) = det_bg {
            return Ok((det_bg.theme(), "Detected background color"));
        }

        clear_screen(Some(&title), color_mode, debug_mode)
            .expect("title should not contain invalid color codes");

        todo!()
    };

    let theme = {
        let (theme, ttl) = select_theme().context("failed to select theme")?;
        debug!(?theme, "selected theme");
        update_title(&mut title, &mut option_counter, ttl, theme.into());
        theme
    };

    //////////////////////////////
    // 3. Choose preset

    // Create flag lines
    let mut flags = Vec::with_capacity(Preset::COUNT);
    let spacing = {
        let Some(spacing) = <Preset as VariantNames>::VARIANTS
            .iter()
            .map(|name| name.chars().count())
            .max()
        else {
            unreachable!();
        };
        let spacing: u8 = spacing.try_into().expect("`spacing` should fit in `u8`");
        cmp::max(spacing, 20)
    };
    for preset in <Preset as VariantArray>::VARIANTS {
        let color_profile = preset.color_profile();
        let flag = color_profile
            .color_text(
                " ".repeat(spacing as usize),
                color_mode,
                ForegroundBackground::Background,
                false,
            )
            .with_context(|| format!("failed to color flag using preset: {preset:?}"))?;
        let name = {
            let name: &'static str = preset.into();
            format!("{name:^spacing$}", spacing = spacing as usize)
        };
        flags.push([name, flag.clone(), flag.clone(), flag]);
    }

    // Calculate flags per row
    let (flags_per_row, rows_per_page) = {
        let (term_w, term_h) = terminal_size().context("failed to get terminal size")?;
        let flags_per_row = term_w.0 / (spacing as u16 + 2);
        let flags_per_row: u8 = flags_per_row
            .try_into()
            .expect("`flags_per_row` should fit in `u8`");
        let rows_per_page = ((term_h.0 - 13) as f32 / 5.0).floor() as usize;
        let rows_per_page: u8 = rows_per_page
            .try_into()
            .expect("`rows_per_page` should fit in `u8`");
        let rows_per_page = cmp::max(1, rows_per_page);
        (flags_per_row, rows_per_page)
    };
    let num_pages = (Preset::COUNT as f32 / (flags_per_row * rows_per_page) as f32).ceil() as usize;
    let num_pages: u8 = num_pages
        .try_into()
        .expect("`num_pages` should fit in `u8`");

    // Create pages
    let mut pages = Vec::with_capacity(num_pages as usize);
    for flags in flags.chunks((flags_per_row * rows_per_page) as usize) {
        let mut page = Vec::with_capacity(rows_per_page as usize);
        for flags in flags.chunks(flags_per_row as usize) {
            page.push(flags);
        }
        pages.push(page);
    }

    let print_flag_row = |row: &[[String; 4]]| {
        for i in 0..4 {
            let mut line = String::new();
            for flag in row {
                line.push_str(&flag[i]);
                line.push_str("  ");
            }
            printc(line, color_mode).expect("flag line should not contain invalid color codes");
        }
        println!();
    };

    let print_flag_page = |page, page_num| {
        clear_screen(Some(&title), color_mode, debug_mode)
            .expect("title should not contain invalid color codes");
        print_title_prompt(option_counter, "Let's choose a flag!", color_mode);
        printc("Available flag presets:", color_mode)
            .expect("prompt should not contain invalid color codes");
        {
            let page_num = page_num + 1;
            println!("Page: {page_num} of {num_pages}");
        }
        println!();
        for &row in page {
            print_flag_row(row);
        }
        println!();
    };

    let preset_rainbow = Preset::Rainbow
        .color_profile()
        .with_lightness_adaptive(Config::default_lightness(theme), theme, use_overlay)
        .color_text(
            "preset",
            color_mode,
            ForegroundBackground::Foreground,
            false,
        )
        .expect("coloring text with rainbow preset should not fail");

    let preset: Preset;
    let color_profile;

    let mut page: u8 = 0;
    loop {
        print_flag_page(&pages[page as usize], page);

        let mut opts = Vec::from(<Preset as VariantNames>::VARIANTS);
        if page < num_pages - 1 {
            opts.push("next");
        }
        if page > 0 {
            opts.push("prev");
        }
        println!("Enter 'next' to go to the next page and 'prev' to go to the previous page.");
        let selection = literal_input(
            format!(
                "Which {preset} do you want to use? ",
                preset = preset_rainbow
            ),
            &opts[..],
            Preset::Rainbow.into(),
            false,
            color_mode,
        )
        .context("failed to select preset")?;
        if selection == "next" {
            page += 1;
        } else if selection == "prev" {
            page -= 1;
        } else {
            preset = selection.parse().expect("selected preset should be valid");
            debug!(?preset, "selected preset");
            color_profile = preset.color_profile();
            update_title(
                &mut title,
                &mut option_counter,
                "Selected flag",
                &color_profile
                    .with_lightness_adaptive(Config::default_lightness(theme), theme, use_overlay)
                    .color_text(
                        <&'static str>::from(preset),
                        color_mode,
                        ForegroundBackground::Foreground,
                        false,
                    )
                    .expect("coloring text with selected preset should not fail"),
            );
            break;
        }
    }

    //////////////////////////////
    // 4. Dim/lighten colors

    let test_ascii = &TEST_ASCII[1..(TEST_ASCII.len() - 1)];
    let Some(test_ascii_width) = test_ascii
        .split('\n')
        .map(|line| line.graphemes(true).count())
        .max()
    else {
        unreachable!();
    };
    let test_ascii_width: u8 = test_ascii_width
        .try_into()
        .expect("`test_ascii_width` should fit in `u8`");
    let test_ascii_height = test_ascii.split('\n').count();
    let test_ascii_height: u8 = test_ascii_height
        .try_into()
        .expect("`test_ascii_height` should fit in `u8`");

    let select_lightness = || -> Result<Lightness> {
        clear_screen(Some(&title), color_mode, debug_mode)
            .expect("title should not contain invalid color codes");
        print_title_prompt(
            option_counter,
            "Let's adjust the color brightness!",
            color_mode,
        );
        printc(
            format!(
                "The colors might be a little bit too {bright_dark} for {light_dark} mode.",
                bright_dark = match theme {
                    TerminalTheme::Light => "bright",
                    TerminalTheme::Dark => "dark",
                },
                light_dark = <&'static str>::from(theme)
            ),
            color_mode,
        )
        .expect("message should not contain invalid color codes");
        println!();

        // Print cats
        {
            let (term_w, _) = terminal_size().context("failed to get terminal size")?;
            let num_cols = cmp::max(1, term_w.0 / (test_ascii_width as u16 + 2));
            let num_cols: u8 = num_cols.try_into().expect("`num_cols` should fit in `u8`");
            const MIN: f32 = 0.15;
            const MAX: f32 = 0.85;
            let ratios =
                (0..num_cols)
                    .map(|col| col as f32 / num_cols as f32)
                    .map(|r| match theme {
                        TerminalTheme::Light => r * (MAX - MIN) / 2.0 + MIN,
                        TerminalTheme::Dark => (r * (MAX - MIN) + (MAX + MIN)) / 2.0,
                    });
            let row: Vec<Vec<String>> = ratios
                .map(|r| {
                    let color_align = ColorAlignment::Horizontal { fore_back: None };
                    let asc = color_align
                        .recolor_ascii(
                            test_ascii.replace(
                                "{txt}",
                                &format!(
                                    "{lightness:^5}",
                                    lightness = format!("{lightness:.0}%", lightness = r * 100.0)
                                ),
                            ),
                            &color_profile.with_lightness_adaptive(
                                Lightness::new(r)
                                    .expect("generated lightness should not be invalid"),
                                theme,
                                use_overlay,
                            ),
                            color_mode,
                            theme,
                        )
                        .expect("recoloring test ascii should not fail");
                    asc.split('\n').map(ToOwned::to_owned).collect::<Vec<_>>()
                })
                .collect();
            for i in 0..(test_ascii_height as usize) {
                let mut line = String::new();
                for lines in &row {
                    line.push_str(&lines[i]);
                    line.push_str("  ");
                }
                printc(line, color_mode)
                    .expect("test ascii line should not contain invalid color codes");
            }
        }

        let default_lightness = Config::default_lightness(theme);

        let parse_lightness = |lightness: String| -> Result<Lightness> {
            if lightness.is_empty() || ["unset", "none"].contains(&&*lightness) {
                return Ok(default_lightness);
            }

            let lightness = if let Some(lightness) = lightness.strip_suffix('%') {
                let lightness: RangedU8<0, 100> = lightness.parse()?;
                lightness.get() as f32 / 100.0
            } else {
                match lightness.parse::<RangedU8<0, 100>>() {
                    Ok(lightness) => lightness.get() as f32 / 100.0,
                    Err(_) => lightness.parse::<f32>()?,
                }
            };

            Ok(Lightness::new(lightness)?)
        };

        loop {
            println!();
            printc(
                format!(
                    "Which brightness level looks the best? (Default: {default:.0}% for \
                     {light_dark} mode)",
                    default = f32::from(default_lightness) * 100.0,
                    light_dark = <&'static str>::from(theme)
                ),
                color_mode,
            )
            .expect("prompt should not contain invalid color codes");
            let lightness = {
                let mut buf = String::new();
                print!("> ");
                io::stdout().flush()?;
                io::stdin()
                    .read_line(&mut buf)
                    .context("failed to read line from input")?;
                buf.trim().to_lowercase()
            };

            match parse_lightness(lightness) {
                Ok(lightness) => {
                    return Ok(lightness);
                },
                Err(err) => {
                    debug!(%err, "could not parse lightness");
                    printc(
                        "&cUnable to parse lightness value, please enter a lightness value such \
                         as 45%, .45, or 45",
                        color_mode,
                    )
                    .expect("message should not contain invalid color codes");
                },
            }
        }
    };

    let lightness = select_lightness().context("failed to select lightness")?;
    debug!(?lightness, "selected lightness");
    let color_profile = color_profile.with_lightness_adaptive(lightness, theme, use_overlay);
    update_title(
        &mut title,
        &mut option_counter,
        "Selected brightness",
        &format!("{lightness:.2}", lightness = f32::from(lightness)),
    );

    //////////////////////////////
    // 5. Color arrangement

    todo!()
}

/// Asks the user to provide an input among a list of options.
fn literal_input<'a, S>(
    prompt: S,
    options: &[&'a str],
    default: &str,
    show_options: bool,
    color_mode: AnsiMode,
) -> Result<&'a str>
where
    S: AsRef<str>,
{
    let prompt = prompt.as_ref();

    if show_options {
        let options_text = options
            .iter()
            .map(|&o| {
                if o == default {
                    format!("&l&n{o}&L&N")
                } else {
                    o.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("|");
        printc(format!("{prompt} ({options_text})"), color_mode)
            .context("failed to print input prompt")?;
    } else {
        printc(format!("{prompt} (default: {default})"), color_mode)
            .context("failed to print input prompt")?;
    }

    let find_selection = |sel: &str| {
        if sel.is_empty() {
            return None;
        }

        // Find exact match
        if let Some(selected) = options.iter().find(|&&o| o.to_lowercase() == sel) {
            return Some(selected);
        }

        // Find starting abbreviation
        if let Some(selected) = options.iter().find(|&&o| o.to_lowercase().starts_with(sel)) {
            return Some(selected);
        }

        None
    };

    loop {
        let mut buf = String::new();
        print!("> ");
        io::stdout().flush()?;
        io::stdin()
            .read_line(&mut buf)
            .context("failed to read line from input")?;
        let selection = {
            let selection = buf.trim_end_matches(&['\r', '\n']);
            if selection.is_empty() {
                default.to_owned()
            } else {
                selection.to_lowercase()
            }
        };

        if let Some(selected) = find_selection(&selection) {
            println!();

            return Ok(selected);
        } else {
            let options_text = options.join("|");
            println!("Invalid selection! {selection} is not one of {options_text}");
        }
    }
}

fn init_tracing_subsriber(debug_mode: bool) -> Result<()> {
    use std::env;
    use std::str::FromStr;

    use tracing::Level;
    use tracing_subscriber::filter::{LevelFilter, Targets};
    use tracing_subscriber::fmt::Subscriber;
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    let builder = Subscriber::builder();

    // Remove the default max level filter from the subscriber; it will be added to
    // the `Targets` filter instead if no filter is set in `RUST_LOG`.
    // Replacing the default `LevelFilter` with an `EnvFilter` would imply this,
    // but we can't replace the builder's filter with a `Targets` filter yet.
    let builder = builder.with_max_level(LevelFilter::TRACE);

    let subscriber = builder.finish();
    let subscriber = {
        let targets = match env::var("RUST_LOG") {
            Ok(var) => Targets::from_str(&var)
                .map_err(|e| {
                    eprintln!("Ignoring `RUST_LOG={var:?}`: {e}");
                })
                .unwrap_or_default(),
            Err(env::VarError::NotPresent) => {
                Targets::new().with_default(Subscriber::DEFAULT_MAX_LEVEL)
            },
            Err(e) => {
                eprintln!("Ignoring `RUST_LOG`: {e}");
                Targets::new().with_default(Subscriber::DEFAULT_MAX_LEVEL)
            },
        };
        let targets = if debug_mode {
            targets.with_target(env!("CARGO_CRATE_NAME"), Level::DEBUG)
        } else {
            targets
        };
        subscriber.with(targets)
    };

    subscriber
        .try_init()
        .context("failed to set the global default subscriber")
}
