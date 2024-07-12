use std::borrow::Cow;
use std::cmp;
use std::fs::{self, File};
use std::io::{self, ErrorKind, IsTerminal, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use hyfetch::cli_options::options;
use hyfetch::color_util::{clear_screen, color, printc, ForegroundBackground, Theme};
use hyfetch::models::Config;
#[cfg(windows)]
use hyfetch::neofetch_util::ensure_git_bash;
use hyfetch::neofetch_util::{self, ascii_size, get_distro_ascii, literal_input, ColorAlignment};
use hyfetch::presets::{AssignLightness, ColorProfile, Preset};
use hyfetch::types::{AnsiMode, LightDark};
use hyfetch::utils::get_cache_path;
use palette::Srgb;
use strum::{EnumCount, VariantArray, VariantNames};
use terminal_size::terminal_size;
use time::{Month, OffsetDateTime};
use tracing::debug;

fn main() -> Result<()> {
    #[cfg(windows)]
    if let Err(err) = enable_ansi_support::enable_ansi_support() {
        debug!(err, "could not enable ANSI escape code support");
    }

    let options = options().run();

    init_tracing_subsriber(options.debug).context("failed to init tracing subscriber")?;

    debug!(?options, "CLI options");

    // Use a custom distro
    let distro = options.distro.as_ref();

    #[cfg(windows)]
    ensure_git_bash().context("failed to find git bash")?;

    if options.test_print {
        let (asc, _) = get_distro_ascii(distro).context("failed to get distro ascii")?;
        println!("{asc}");
        return Ok(());
    }

    let config = if options.config {
        create_config(distro, &options.config_file, options.overlay, options.debug)
            .context("failed to create config")?
    } else if let Some(config) =
        read_config(&options.config_file).context("failed to read config")?
    {
        config
    } else {
        create_config(distro, &options.config_file, options.overlay, options.debug)
            .context("failed to create config")?
    };

    // Check if it's June (pride month)
    let now =
        OffsetDateTime::now_local().context("failed to get current datetime in local timezone")?;
    let cache_path = get_cache_path().context("failed to get cache path")?;
    let june_path = cache_path.join(format!("animation-displayed-{}", now.year()));
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

    let color_mode = options.mode.unwrap_or(config.mode);
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
        color_profile.with_lightness_dl(config.lightness(), config.light_dark, options.overlay)
    };
    debug!(?color_profile, "lightened color profile");

    let (asc, fore_back) = if let Some(path) = options.ascii_file {
        (
            fs::read_to_string(&path)
                .with_context(|| format!("failed to read ascii from {path:?}"))?,
            None,
        )
    } else {
        get_distro_ascii(distro).context("failed to get distro ascii")?
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
        .recolor_ascii(asc, color_profile, color_mode, config.light_dark)
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
        Err(err) if err.kind() == ErrorKind::NotFound => {
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
    distro: Option<&String>,
    path: &Path,
    use_overlay: bool,
    debug_mode: bool,
) -> Result<Config> {
    // Detect terminal environment (doesn't work for all terminal emulators,
    // especially on Windows)
    let det_bg = match termbg::rgb(std::time::Duration::from_millis(100)) {
        Ok(rgb) => Some(Srgb::<u16>::new(rgb.r, rgb.g, rgb.b).into_format::<u8>()),
        Err(termbg::Error::Unsupported) => None,
        Err(err) => {
            return Err(err).context("failed to get background color");
        },
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

    let (asc, fore_back) = get_distro_ascii(distro).context("failed to get distro ascii")?;
    let (asc_width, asc_lines) = ascii_size(asc);
    let theme = det_bg.map(|bg| bg.theme()).unwrap_or(LightDark::Light);
    let color_mode = det_ansi.unwrap_or(AnsiMode::Ansi256);
    let logo = color(
        match theme {
            LightDark::Light => "&l&bhyfetch&~&L",
            LightDark::Dark => "&l&bhy&ffetch&~&L",
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
        title.push_str({
            let pad = " ".repeat(30 - k.len());
            &format!("\n&e{option_counter}. {k}{pad} &~{v}")
        });
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
    // 1. Select color system

    let select_color_system = || -> Result<(AnsiMode, &str)> {
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
        let (color_system, ttl) = select_color_system().context("failed to select color system")?;
        debug!(?color_system, "selected color mode");
        update_title(&mut title, &mut option_counter, ttl, color_system.into());
        color_system
    };

    //////////////////////////////
    // 2. Select light/dark mode

    let select_light_dark = || -> Result<(LightDark, &str)> {
        if let Some(det_bg) = det_bg {
            return Ok((det_bg.theme(), "Detected background color"));
        }

        clear_screen(Some(&title), color_mode, debug_mode)
            .expect("title should not contain invalid color codes");

        todo!()
    };

    let theme = {
        let (light_dark, ttl) =
            select_light_dark().context("failed to select light / dark mode")?;
        debug!(?light_dark, "selected theme");
        update_title(&mut title, &mut option_counter, ttl, light_dark.into());
        light_dark
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
            let name_len = name.chars().count();
            let name_len: u8 = name_len.try_into().expect("`name_len` should fit in `u8`");
            let pad_start = " ".repeat(((spacing - name_len) / 2) as usize);
            let pad_end =
                " ".repeat(((spacing - name_len) / 2 + (spacing - name_len) % 2) as usize);
            format!("{pad_start}{name}{pad_end}")
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

    let color_profile: ColorProfile;
    let preset_rainbow = Preset::Rainbow
        .color_profile()
        .with_lightness_dl(Config::default_lightness(theme), theme, use_overlay)
        .color_text(
            "preset",
            color_mode,
            ForegroundBackground::Foreground,
            false,
        )
        .expect("coloring text with rainbow preset should not fail");

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
        let preset = literal_input(
            format!("Which {preset_rainbow} do you want to use? "),
            &opts[..],
            Preset::Rainbow.into(),
            false,
            color_mode,
        )
        .context("failed to select preset")?;
        if preset == "next" {
            page += 1;
        } else if preset == "prev" {
            page -= 1;
        } else {
            let preset: Preset = preset.parse().expect("selected preset should be valid");
            debug!(?preset, "selected preset");
            color_profile = preset.color_profile();
            {
                let preset_name: &'static str = preset.into();
                let preset_colored_name = color_profile
                    .with_lightness_dl(Config::default_lightness(theme), theme, use_overlay)
                    .color_text(
                        preset_name,
                        color_mode,
                        ForegroundBackground::Foreground,
                        false,
                    )
                    .expect("coloring text with selected preset should not fail");
                update_title(
                    &mut title,
                    &mut option_counter,
                    "Selected flag",
                    &preset_colored_name,
                );
            }
            break;
        }
    }

    //////////////////////////////
    // 4. Dim/lighten colors

    // TODO

    todo!()
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
