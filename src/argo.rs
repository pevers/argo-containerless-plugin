use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PluginParameters {
    // The git repo url, must be HTTPS
    #[serde(rename = "repoURL")]
    pub repo_url: String,

    // The runtime, python-3.10.0 for example
    pub runtime: String,

    // The repo target revision, can be a branch or commit hash
    #[serde(rename = "targetRevision")]
    pub target_revision: String,

    // The arguments passed to the program
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Plugin {
    pub containerless: PluginParameters,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Metadata {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Template {
    pub name: String,
    pub plugin: Plugin,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Workflow {
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Outputs {
    #[serde(rename = "exitCode")]
    pub exit_code: Option<String>,
    pub result: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct NodeResult {
    pub phase: String,
    pub message: String,
    pub outputs: Option<Outputs>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ExecuteTemplateArgs {
    pub template: Template,
    pub workflow: Workflow,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ExecuteTemplateReply {
    pub node: NodeResult,
}

pub fn error_response(msg: &str) -> ExecuteTemplateReply {
    ExecuteTemplateReply {
        node: NodeResult {
            phase: "Error".into(),
            message: msg.into(),
            outputs: Some(Outputs {
                exit_code: Some("1".into()),
                result: None,
            }),
        },
    }
}

pub fn ok_response(
    msg: &str,
    exit_code: Option<String>,
    stdout: Option<String>,
) -> ExecuteTemplateReply {
    ExecuteTemplateReply {
        node: NodeResult {
            phase: "Succeeded".into(),
            message: msg.into(),
            outputs: Some(Outputs {
                exit_code,
                result: stdout,
            }),
        },
    }
}
