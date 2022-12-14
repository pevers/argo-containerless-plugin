//! Install, build and run a Node project from a git reference

use crate::argo::PluginParameters;
use git2::Repository;
use regex::Regex;
use snafu::prelude::*;
use uuid::Uuid;

use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Cannot parse Git reference, we only support HTTPS at the moment"))]
    InvalidGitReference,

    #[snafu(display("Git operation failed"))]
    GitFailed { source: git2::Error },
}

pub struct ModuleInfo<'a> {
    // The module host, "github.com" for example
    pub host: &'a str,
    // The repo, "promaton" for example
    pub repo: &'a str,
    // The module, "example-project" for example
    pub module: &'a str,
}

fn extract_module_info<'a>(plugin: &'a PluginParameters) -> Result<ModuleInfo<'a>, Error> {
    let re = Regex::new(r"https://(?P<host>.*)/(?P<repo>.*)/(?P<module>.*).git").unwrap();
    Ok(re
        .captures(&plugin.repo_url)
        .map(|captures| {
            let host = captures.name("host")?;
            let repo = captures.name("repo")?;
            let module = captures.name("module")?;
            return Some(ModuleInfo {
                host: host.as_str(),
                repo: repo.as_str(),
                module: module.as_str(),
            });
        })
        .flatten()
        .context(InvalidGitReferenceSnafu)?)
}

// Checkout specific branch, tag or commit
fn checkout_branch(target_dir: &PathBuf, refname: &str) -> Result<(), Error> {
    let repo = Repository::open(target_dir).context(GitFailedSnafu)?;
    let (object, reference) = repo.revparse_ext(refname).context(GitFailedSnafu)?;
    repo.checkout_tree(&object, None).context(GitFailedSnafu)?;
    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()).context(GitFailedSnafu),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()).context(GitFailedSnafu),
    }
}

// Clones the repo to the target directory.
// The controller needs to have an access token defined in the secrets if the repo is private.
fn clone_repo(plugin: &PluginParameters, target_dir: &PathBuf) -> Result<Repository, Error> {
    info!("Cloning {:?} into {:?}", plugin, target_dir);
    Repository::clone(&plugin.repo_url, target_dir).context(GitFailedSnafu)
}

// Clone the module to a unique location and return the target location
pub fn clone_module(plugin: &PluginParameters) -> Result<PathBuf, Error> {
    let workdir = env::var("PYTHON_WORKDIR").unwrap_or(env::temp_dir().display().to_string());
    let unique_id = Uuid::new_v4().to_string();
    let target_dir = Path::new(&workdir).join(&unique_id);
    clone_repo(plugin, &target_dir)?;
    checkout_branch(&target_dir, &plugin.target_revision)?;
    Ok(target_dir)
}

#[cfg(test)]
mod tests {
    use super::extract_module_info;
    use crate::{argo::PluginParameters, git::clone_module};
    use std::fs;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_extract_module_info() {
        let parameters = PluginParameters {
            repo_url: "https://github.com/promaton/some-repo.git".to_string(),
            runtime: "node-16.13".to_string(),
            target_revision: "main".to_string(),
            args: vec!["build/example.js".into()],
        };
        let module_info = extract_module_info(&parameters).unwrap();
        assert_eq!(module_info.host, "github.com");
        assert_eq!(module_info.repo, "promaton");
        assert_eq!(module_info.module, "some-repo");
    }

    #[test]
    fn test_clone_module() {
        init();
        let parameters = PluginParameters {
            repo_url: "https://github.com/pevers/images-scraper.git".to_string(),
            runtime: "node-16.13".to_string(),
            target_revision: "master".to_string(),
            args: vec!["build/example.js".into()],
        };
        let target_dir = clone_module(&parameters).unwrap();

        // Directory is created
        assert_eq!(fs::metadata(target_dir).unwrap().is_dir(), true);
    }
}
