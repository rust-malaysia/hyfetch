use anyhow::{Context, Result};
use hyfetch::cli_options::options;
use hyfetch::neofetch_util::get_distro_ascii;
use log::debug;

fn main() -> Result<()> {
    env_logger::init();

    let options = options().fallback_to_usage().run();
    debug!(options:?; "CLI options");

    // TODO

    if options.test_print {
        println!(
            "{}",
            get_distro_ascii(None).context("Failed to get distro ascii")?
        );
        return Ok(());
    }

    // TODO

    Ok(())
}
