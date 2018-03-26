
use std::path::Path;

use git2::{Repository, Error, RepositoryState, MergeAnalysis};
use git2::{FetchOptions, RemoteCallbacks, CredentialType};

pub fn get_head_sha(path: &Path) -> Result<String, Error> {
    let repo = Repository::open(path)?;
    let obj = repo.revparse_single("HEAD^{commit}")?;
    let oid = obj.id();

    Ok(format!("{}", oid))
}

// update origin and merge HEAD onto master
// will only perform fast forward merges on a clean repo
pub fn pull(path: &Path, ) -> Result<(), Error> {
    let repo = Repository::open(path)?;

    if repo.state() != RepositoryState::Clean {
        return Err(Error::from_str("repository is not clean"));
    }
    
    // add callback to manage credentials
    let callbacks = RemoteCallbacks::new()
        .credentials(|_url: &str, _username: Option<&str>, cred_type: CredentialType| {
        });
    let opts = FetchOptions::new().remote_callbacks(callbacks);

    // fetch new commits
    repo.find_remote("origin")?.fetch(&["master"], None, None)?;

    // get commit for remote master
    let oid = repo.revparse_single("origin/master^{commit}")?.id();
    let r_head = repo.find_annotated_commit(oid)?;

    match repo.merge_analysis(&[&r_head])? {
        (MergeAnalysis::ANALYSIS_FASTFORWARD, _) => {},
        (MergeAnalysis::ANALYSIS_UP_TO_DATE, _) => {
            // no merge necessary
            return Ok(());
        },
        _ => return Err(Error::from_str("cannot continue with merge, fastforward not possible")),
    }

    repo.merge(&[&r_head], None, None)?;

    Ok(())
}

