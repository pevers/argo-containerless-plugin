//! Manages Python environments by using `pyenv` under the hood
use regex::Regex;
use snafu::prelude::*;
use std::{
    path::PathBuf,
    process::{Command, Output},
};

use crate::{
    argo::{ok_response, ExecuteTemplateArgs, ExecuteTemplateReply},
    git::clone_module,
    runner::{self, RuntimeExecutor},
};

const MINIMUM_MAJOR_VERSION: i32 = 3;
const MINIMUM_MINOR_VERSION: i32 = 7;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "Invalid Python version: {}, must be >= {}.{}!",
        version,
        MINIMUM_MAJOR_VERSION,
        MINIMUM_MINOR_VERSION
    ))]
    InvalidPythonVersion { version: String },

    #[snafu(display("Invalid command"))]
    InvalidCommand { source: std::io::Error },

    #[snafu(display("Command failed with: {}, {}", stdout, stderr))]
    CommandFailed { stdout: String, stderr: String },
}

#[derive(Debug, PartialEq)]
pub struct PythonRuntime {
    // The Python version (3.10 for example)
    pub version: String,
}

impl PythonRuntime {
    pub fn new(version: &str) -> Result<Self, Error> {
        let re = Regex::new(r"(?P<major>\d+)\.(?P<minor>\d+)").unwrap();
        re.captures(version)
            .map(|captures| {
                let major = captures.name("major")?.as_str().parse::<i32>().ok()?;
                let minor = captures.name("minor")?.as_str().parse::<i32>().ok()?;
                if major != MINIMUM_MAJOR_VERSION || minor < MINIMUM_MINOR_VERSION {
                    return None;
                }
                return Some({
                    PythonRuntime {
                        version: version.to_owned(),
                    }
                });
            })
            .flatten()
            .ok_or(Error::InvalidPythonVersion {
                version: version.to_owned(),
            })
    }
}

// Creates a new virtual environment and installs all dependencies for a given path.
// Poetry must be installed for this to work.
fn install_dependencies(path: &PathBuf) -> Result<(), Error> {
    info!("Installing dependencies for {:?}", path);
    Command::new("poetry")
        .current_dir(path)
        .args(["install"])
        .output()
        .context(InvalidCommandSnafu)
        .and_then(|output| {
            if !output.status.success() {
                return CommandFailedSnafu {
                    stdout: String::from_utf8(output.stdout).unwrap(),
                    stderr: String::from_utf8(output.stderr).unwrap(),
                }
                .fail();
            }
            Ok(())
        })
}

// Runs a given script in the Python project via Poetry
// and returns the output.
fn run_command(path: &PathBuf, arguments: &Vec<String>) -> Result<Output, Error> {
    info!(
        "Running Python with arguments `{:?}` in path {:?}",
        arguments, path
    );
    let output = Command::new("poetry")
        .current_dir(path)
        .args(["run", "python"])
        .args(arguments)
        .output()
        .context(InvalidCommandSnafu)?;
    Ok(output)
}

impl RuntimeExecutor for PythonRuntime {
    // Execute the given Argo Workflow request
    fn handle_request(
        &self,
        req: &ExecuteTemplateArgs,
    ) -> Result<ExecuteTemplateReply, runner::Error> {
        let destination =
            clone_module(&req.template.plugin.containerless).context(runner::GitFailedSnafu)?;
        install_dependencies(&destination).context(runner::PythonOperationFailedSnafu)?;
        let output = run_command(&destination, &req.template.plugin.containerless.args)
            .context(runner::PythonOperationFailedSnafu)?;
        Ok(ok_response(
            "Succes", // TODO: Should this contain the stdout?
            output.status.code().map(|code| code.to_string()),
            String::from_utf8(output.stdout).ok(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{install_dependencies, run_command, PythonRuntime};
    use std::path::PathBuf;

    // WARNING: These tests only run in the appropriate environment

    #[test]
    fn test_create_runtime() {
        PythonRuntime::new("3.9").unwrap();
    }

    #[test]
    fn test_create_unsupported_runtime_s() {
        let runtime = PythonRuntime::new("3.6");
        assert_eq!(runtime.is_err(), true);
        if let Err(e) = runtime {
            assert_eq!(
                format!("{}", e),
                "Invalid Python version: 3.6, must be >= 3.7!"
            );
        }
    }

    #[test]
    fn test_install_dependencies() {
        install_dependencies(&PathBuf::from("./test/python/example-project")).unwrap();
    }

    #[test]
    fn test_run_command() {
        let output = run_command(
            &PathBuf::from("./test/python/example-project"),
            &vec![
                "src/example.py".into(),
                "--another".into(),
                "parameter".into(),
            ],
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "The input is ['src/example.py', '--another', 'parameter'], output: 2.0"
        );
    }
}
