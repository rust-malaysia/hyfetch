use anyhow::{Context, Result};
use hyfetch::cli_options::options;
use hyfetch::neofetch_util::get_distro_ascii;
use tracing::debug;

fn main() -> Result<()> {
    let options = options().fallback_to_usage().run();

    init_tracing_subsriber(options.debug).context("Failed to init tracing subscriber")?;

    debug!(?options, "CLI options");

    // TODO

    if options.test_print {
        println!(
            "{}",
            get_distro_ascii(options.distro.as_ref()).context("Failed to get distro ascii")?
        );
        return Ok(());
    }

    // TODO

    Ok(())
}

fn init_tracing_subsriber(debug: bool) -> Result<()> {
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
                    eprintln!("Ignoring `RUST_LOG={:?}`: {}", var, e);
                })
                .unwrap_or_default(),
            Err(env::VarError::NotPresent) => {
                let targets = Targets::new().with_default(Subscriber::DEFAULT_MAX_LEVEL);
                if debug {
                    targets.with_target(env!("CARGO_CRATE_NAME"), Level::DEBUG)
                } else {
                    targets
                }
            },
            Err(e) => {
                eprintln!("Ignoring `RUST_LOG`: {}", e);
                Targets::new().with_default(Subscriber::DEFAULT_MAX_LEVEL)
            },
        };
        subscriber.with(targets)
    };

    subscriber
        .try_init()
        .context("Failed to set the global default subscriber")
}
