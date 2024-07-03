use std::borrow::Cow;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::{env, fmt};

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::color_util::{NeofetchAsciiIndexedColor, PresetIndexedColor};
use crate::distros::Distro;
use crate::presets::ColorProfile;
use crate::types::{AnsiMode, Backend, LightDark};

const NEOFETCH_COLOR_PATTERN: &str = r"\$\{c[0-6]\}";
static NEOFETCH_COLOR_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "mode")]
#[serde(rename_all = "lowercase")]
pub enum ColorAlignment {
    Horizontal {
        fore_back: Option<(NeofetchAsciiIndexedColor, NeofetchAsciiIndexedColor)>,
    },
    Vertical {
        fore_back: Option<(NeofetchAsciiIndexedColor, NeofetchAsciiIndexedColor)>,
    },
    Custom {
        #[serde(rename = "custom_colors")]
        colors: IndexMap<NeofetchAsciiIndexedColor, PresetIndexedColor>,
    },
}

impl ColorAlignment {
    /// Uses the color alignment to recolor an ascii art.
    pub fn recolor_ascii(
        &self,
        asc: String,
        color_profile: ColorProfile,
        color_mode: AnsiMode,
        term: LightDark,
    ) -> String {
        todo!()
    }
}

/// Gets the absolute path of the neofetch command.
pub fn get_command_path() -> Result<PathBuf> {
    if let Ok(workspace_dir) = env::var("CARGO_WORKSPACE_DIR") {
        let path = Path::new(&workspace_dir);
        if path.exists() {
            let path = path.join("neofetch");
            match path.try_exists() {
                Ok(true) => {
                    return path.canonicalize().context("failed to canonicalize path");
                },
                Ok(false) => {
                    Err(anyhow!("{path:?} does not exist or is not readable"))?;
                },
                Err(err) => {
                    Err(err)
                        .with_context(|| format!("failed to check for existence of {path:?}"))?;
                },
            }
        }
    }

    let Ok(path_env) = env::var("PATH") else {
        return Err(anyhow!("`PATH` env var is not set or invalid"));
    };

    for search_path in env::split_paths(&path_env) {
        let path = search_path.join("neowofetch");
        if !path.is_file() {
            continue;
        }
        return path.canonicalize().context("failed to canonicalize path");
    }

    Err(anyhow!("neofetch command not found"))
}

/// Gets the distro ascii of the current distro. Or if distro is specified, get
/// the specific distro's ascii art instead.
#[tracing::instrument(level = "debug")]
pub fn get_distro_ascii<S>(distro: Option<S>) -> Result<String>
where
    S: AsRef<str> + fmt::Debug,
{
    let distro: Cow<_> = if let Some(distro) = distro.as_ref() {
        distro.as_ref().into()
    } else {
        get_distro_name()
            .context("failed to get distro name")?
            .into()
    };
    debug!(%distro, "distro name");

    if let Some(distro) = Distro::detect(&distro) {
        return Ok(normalize_ascii(distro.ascii_art()));
    }

    todo!()
}

pub fn run(asc: String, backend: Backend, args: Option<&Vec<String>>) -> Result<()> {
    todo!()
}

/// Gets distro ascii width and height, ignoring color code.
pub fn ascii_size<S>(asc: S) -> (u8, u8)
where
    S: AsRef<str>,
{
    let asc = asc.as_ref();

    let Some(width) = NEOFETCH_COLOR_RE
        .get_or_init(|| Regex::new(NEOFETCH_COLOR_PATTERN).unwrap())
        .replace_all(asc, "")
        .split('\n')
        .map(|line| line.len())
        .max()
    else {
        unreachable!();
    };
    let height = asc.split('\n').count();

    (width as u8, height as u8)
}

/// Makes sure every line are the same width.
fn normalize_ascii<S>(asc: S) -> String
where
    S: AsRef<str>,
{
    let asc = asc.as_ref();

    let (w, _) = ascii_size(asc);

    let mut buf = String::new();
    for line in asc.split('\n') {
        let (line_w, _) = ascii_size(line);
        let pad = " ".repeat((w - line_w) as usize);
        buf.push_str(&format!("{line}{pad}\n"))
    }

    buf
}

/// Runs neofetch command, returning the piped stdout output.
#[tracing::instrument(level = "debug")]
fn run_neofetch_command_piped<S>(args: &[S]) -> Result<String>
where
    S: AsRef<OsStr> + fmt::Debug,
{
    let mut command = make_neofetch_command(args).context("failed to make neofetch command")?;

    let output = command
        .output()
        .context("failed to execute neofetch as child process")?;
    debug!(?output, "neofetch output");

    if !output.status.success() {
        let err = if let Some(code) = output.status.code() {
            anyhow!("neofetch process exited with status code: {code}")
        } else {
            #[cfg(unix)]
            {
                anyhow!(
                    "neofetch process terminated by signal: {}",
                    output
                        .status
                        .signal()
                        .expect("either one of status code or signal should be set")
                )
            }
            #[cfg(not(unix))]
            unimplemented!("status code not expected to be `None` on non-Unix platforms")
        };
        Err(err)?;
    }

    let out = String::from_utf8(output.stdout)
        .context("failed to process neofetch output as it contains invalid UTF-8")?
        .trim()
        .to_owned();
    Ok(out)
}

fn make_neofetch_command<S>(args: &[S]) -> Result<Command>
where
    S: AsRef<OsStr>,
{
    #[cfg(not(windows))]
    {
        let mut command = Command::new("bash");
        command.arg(get_command_path().context("failed to get neofetch command path")?);
        command.args(args);
        Ok(command)
    }
    #[cfg(windows)]
    {
        todo!()
    }
}

#[tracing::instrument(level = "debug")]
fn get_distro_name() -> Result<String> {
    run_neofetch_command_piped(&["ascii_distro_name"])
        .context("failed to get distro name from neofetch")
}
