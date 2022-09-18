use crate::{
    argo::{ExecuteTemplateArgs, ExecuteTemplateReply},
    git,
    python::{self, PythonRuntime},
};
use snafu::{prelude::*, Backtrace};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Git operation failed"), visibility(pub))]
    GitFailed {
        source: git::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Python operations failed"), visibility(pub))]
    PythonOperationFailed {
        source: python::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Runtime not found {}", runtime))]
    RuntimeNotFound { runtime: String },
}

pub trait RuntimeExecutor {
    fn handle_request(&self, req: &ExecuteTemplateArgs) -> Result<ExecuteTemplateReply, Error>;
}

// Extract the runtime from the request
pub fn extract_runtime(req: &ExecuteTemplateArgs) -> Result<Box<dyn RuntimeExecutor>, Error> {
    let chunks: Vec<&str> = req
        .template
        .plugin
        .containerless
        .runtime
        .split("-")
        .collect();
    let runtime = chunks[0].trim().to_lowercase();
    let version = chunks[1].trim().to_lowercase();
    match runtime.as_str() {
        "node" => unimplemented!(),
        "python" => {
            let rt = PythonRuntime::new(&version).context(PythonOperationFailedSnafu)?;
            Ok(Box::new(rt))
        }
        _ => RuntimeNotFoundSnafu { runtime }.fail(),
    }
}

#[cfg(test)]
mod tests {
    use crate::argo::{
        ExecuteTemplateArgs, Metadata, Plugin, PluginParameters, Template, Workflow,
    };

    use super::extract_runtime;

    #[test]
    fn test_extract_python() {
        let req = ExecuteTemplateArgs {
            template: Template {
                name: String::from("test-module"),
                plugin: Plugin {
                    containerless: PluginParameters {
                        repo_url: "https://github.com/some/repo.git".to_string(),
                        runtime: "python-3.10".to_string(),
                        target_revision: "master".to_string(),
                        args: vec!["src/example.py".into()],
                    },
                },
            },
            workflow: Workflow {
                metadata: Metadata {
                    name: String::from("test-module"),
                },
            },
        };
        extract_runtime(&req).unwrap();
    }

    #[test]
    fn test_unsupported() {
        let req = ExecuteTemplateArgs {
            template: Template {
                name: String::from("test-module"),
                plugin: Plugin {
                    containerless: PluginParameters {
                        repo_url: "https://github.com/some/repo.git".to_string(),
                        runtime: "brainfuck-3.10".to_string(),
                        target_revision: "master".to_string(),
                        args: vec!["src/example.py".into()],
                    },
                },
            },
            workflow: Workflow {
                metadata: Metadata {
                    name: String::from("test-module"),
                },
            },
        };
        let runtime = extract_runtime(&req);
        assert_eq!(runtime.is_err(), true);
    }
}
