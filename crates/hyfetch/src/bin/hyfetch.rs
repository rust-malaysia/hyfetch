use anyhow::Result;
use hyfetch::cli_options::options;
use log::debug;

fn main() -> Result<()> {
    env_logger::init();

    let options = options().fallback_to_usage().run();
    debug!(options:?; "CLI options");

    // TODO

    Ok(())
}
