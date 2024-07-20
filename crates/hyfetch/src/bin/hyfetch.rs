use std::fs::{self, File};
use std::io::{self, IsTerminal as _};

use anyhow::{Context as _, Result};
use hyfetch::cli_options::options;
use hyfetch::models::Config;
#[cfg(windows)]
use hyfetch::neofetch_util::ensure_git_bash;
use hyfetch::neofetch_util::{self, get_distro_ascii, ColorAlignment};
use hyfetch::presets::AssignLightness;
use hyfetch::types::Backend;
use hyfetch::utils::{get_cache_path, input};
use time::{Month, OffsetDateTime};
use tracing::debug;

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
        Config::create_config(
            &options.config_file,
            distro,
            backend,
            use_overlay,
            debug_mode,
        )
        .context("failed to create config")?
    } else if let Some(config) =
        Config::read_from_file(&options.config_file).context("failed to read config")?
    {
        config
    } else {
        Config::create_config(
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
            ColorAlignment::Horizontal { .. } => ColorAlignment::Horizontal { fore_back },
            ColorAlignment::Vertical { .. } => ColorAlignment::Vertical { fore_back },
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
        input(Some("Press enter to exit...")).context("failed to read input")?;
    }

    Ok(())
}

fn init_tracing_subsriber(debug_mode: bool) -> Result<()> {
    use std::env;
    use std::str::FromStr as _;

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
