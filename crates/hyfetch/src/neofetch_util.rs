use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt, process};

use anyhow::{anyhow, Context, Result};
use tracing::debug;
use pyo3::{Python, PyDict, PyString};

use crate::distros::Distro;
use crate::types::{Backend, ColorAlignMode};

/// Ask the user to provide an input among a list of options
/// 
/// prompt: Input prompt
/// options: Options
/// default: Default option
/// show_ops: Show options
/// return: Selection
pub fn literal_input<T>(prompt: String, options: impl IntoIterator<Item = String>, default: String, show_ops: bool) -> Result<String, >
where
    T: std::iter::Iterator
{
    let options = Vec::from_iter(options);
    
    if (show_ops) {
        // let op_text = options.join("|")
        // TODO: printc function from color_util
    } else {
        // TODO: printc function from color_util
    }
    let option_text = options.join("|");

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    if (selection.is_empty()) {
        selection = default;
    }

    let mut lows = options.into_iter().map(|s| s.to_lowercase());

    loop {
        let sel = lows
            .find(|x| x == &selection || x.starts_with(&selection));
        
        match sel {
            None => {
                println!("Invalid selection! {selection} is not one of {option_text}");
                io::stdin().read_line(&mut selection)?;
            },
            Some(s) => return Ok(s)
        }
    }
}

pub fn term_size() -> io::Result<(u16, u16)> {
    termion::terminal_size()
}

/// Get distro ascii width, height ignoring color code
pub fn ascii_size(asc: String) -> (u16, u16) {
    todo!()
    // return (cmp::max(v1), asc.split("\n").clone().count()));
}

/// Make sure every line are the same width
pub fn normalize_ascii(asc: String) -> String {
    let w = ascii_size(asc).0;
    let lines = asc.split("\n").into_iter().map(|line| format!("{:w}", line));
    return lines.join("\n");
}

/// Fill the missing starting placeholders.
pub fn fill_starting(asc: String) -> String {
    todo!()
}

/// Return the file if the file exists, or return none. Useful for chaining 'or's
pub fn if_file(f: &str) -> Option<PathBuf> {
    let file = PathBuf::from(f);
    if file.is_file() {
        return Some(file);
    }
    return None;
}

struct ColorAlignment {
    mode: ColorAlignMode
    custom_colors: HashMap<int, int>
    fore_back: (u16, u16)
}

impl ColorAlignment {
    fn recolor_ascii
}

/// Gets the absolute path of the neofetch command.
pub fn get_command_path() -> Result<PathBuf> {
    if let Ok(workspace_dir) = env::var("CARGO_WORKSPACE_DIR") {
        let path = Path::new(&workspace_dir);
        if path.exists() {
            let path = path.join("neofetch");
            match path.try_exists() {
                Ok(true) => {
                    return path.canonicalize().context("Failed to canonicalize path");
                },
                Ok(false) => {
                    Err(anyhow!("{path:?} does not exist or is not readable"))?;
                },
                Err(err) => {
                    Err(err)
                        .with_context(|| format!("Failed to check for existence of {path:?}"))?;
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
        return path.canonicalize().context("Failed to canonicalize path");
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
            .context("Failed to get distro name")?
            .into()
    };
    debug!(%distro, "distro name");

    if let Some(distro) = Distro::detect(&distro) {
        return Ok(distro.ascii_art().to_owned());
    }

    todo!()
}

/// Runs neofetch command, returning the piped stdout output.
#[tracing::instrument(level = "debug")]
fn run_neofetch_command_piped<S>(args: &[S]) -> Result<String>
where
    S: AsRef<OsStr> + fmt::Debug,
{
    let mut command = make_neofetch_command(args).context("Failed to make neofetch command")?;

    let output = command
        .output()
        .context("Failed to execute neofetch as child process")?;
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
        .context("Failed to process neofetch output as it contains invalid UTF-8")?
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
        command.arg(get_command_path().context("Failed to get neofetch command path")?);
        command.args(args);
        Ok(command)
    }
    #[cfg(windows)]
    {
        todo!()
    }
}

fn run_qwqfetch(asc: &str) -> Result<String> {
    Ok(Python::with_gil(|py| {
        let locals = PyDict::new_bound(py);
        py.run_bound(r#"
        try:
            import qwqfetch
            # distro_detector only return a bash variable
            # so we use qwqfetch builtin distro detector
            ret = qwqfetch.get_ascres(asc)
        except ImportError as e:  # module not found etc
            print("qwqfetch is not installed. Install it by executing:")  # use print to output hint directly
            print("pip install git+https://github.com/nexplorer-3e/qwqfetch")  # TODO: public repo
            exit(127)
        "#, None, None).unwrap();
        let ret = locals.get_item("ret").unwrap().unwrap();
        let ret = ret.downcast::<PyString>().unwrap();
        ret.to_string_lossy().to_owned()
    }))
}

fn run_neofetch_rs() -> Result<String> {
    Ok(neofetch::neofetch())
}

fn fastfetch_path() -> Option<PathBuf> {
    which::which("fastfetch").ok()
        .or(if_file("fastfetch/usr/bin/fastfetch"))
        .or(if_file("fastfetch/fastfetch"))
        .or(if_file("fastfetch/fastfetch.exe"))
}

fn run_fastfetch(asc: &str, args: &str, legacy: bool) -> Result<String> {
    let Some(ff_path) = fastfetch_path() else {
        eprintln!("Error: fastfetch binary is not found. Please install fastfetch first.");
        process::exit(127);
    };

    // Create a temporary directory
    let tmp_dir = tempfile::tempdir().expect("Failed to create temporary directory");

    // Define the path for the temporary file
    let path: PathBuf = tmp_dir.path().join("ascii.txt");

    // Write the ASCII text to the temporary file
    {
        let mut file = File::create(&path).expect("Failed to create file");
        file.write_all(asc.as_bytes()).expect("Failed to write to file");
    }

    // Prepare the fastfetch command arguments
    let mut command = Command::new(ff_path);
    if legacy {
        command.arg("--raw");
    } else {
        command.arg("--file-raw");
    }
    command.arg(path.to_str().unwrap());
    command.args(shlex::split(args).unwrap()); // Assuming you will handle the args parsing similarly

    // Run the fastfetch command
    let output = command.output().expect("Failed to execute fastfetch");

    // Check the return code and print the error message if needed
    if output.status.code() == Some(144) {
        eprintln!("Error code 144 detected: Please upgrade fastfetch to >=1.8.0 or use the 'fastfetch-old' backend");
    }
    let out = String::from_utf8(output.stdout)
        .context("Failed to process neofetch output as it contains invalid UTF-8")?
        .trim()
        .to_owned();
    Ok(out)
}

fn run(asc: &str, backend: Backend, args: &str) -> Result<String> {
    match backend {
        Backend::Qwqfetch => run_qwqfetch(asc),
        Backend::Neofetch => todo!("run_neofetch_command_piped(args)"),
        Backend::NeofetchRs => run_neofetch_rs(),
        Backend::Fastfetch => run_fastfetch(asc, args, false),
        Backend::FastfetchOld => run_fastfetch(asc, args, true),
    }
}

#[tracing::instrument(level = "debug")]
fn get_distro_name() -> Result<String> {
    run_neofetch_command_piped(&["ascii_distro_name"])
        .context("Failed to get distro name from neofetch")
}
