
use std::path::Path;

use git2::{self, Repository, Error, RepositoryState, MergeAnalysis};
use git2::{FetchOptions, RemoteCallbacks, CredentialType, Cred};

pub fn get_head_sha(path: &Path) -> Result<String, Error> {
    let repo = Repository::open(path)?;
    let obj = repo.revparse_single("HEAD^{commit}")?;
    let oid = obj.id();

    Ok(format!("{}", oid))
}

// update origin and merge HEAD onto master
// will only perform fast forward merges on a clean repo
pub fn pull(path: &Path, ssh: &SSHConf) -> Result<(), Error> {
    let repo = Repository::open(path)?;

    if repo.state() != RepositoryState::Clean {
        return Err(Error::from_str("repository is not clean"));
    }
    
    // add callback to manage credentials
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url: &str, username: Option<&str>, cred_type: CredentialType| {
            get_creds(url, username, cred_type, ssh)
    });
    let mut opts = FetchOptions::new();
    opts.remote_callbacks(callbacks);

    // fetch new commits
    repo.find_remote("origin")?
        .fetch(/* empty results in updating all refspecs */ &[], Some(&mut opts), None)?;

    // get commit for remote master
    let oid = repo.revparse_single("origin/master^{commit}")?.id();
    let r_head = repo.find_annotated_commit(oid)?;

    let (analysis, _) = repo.merge_analysis(&[&r_head])?;
    if analysis.contains(MergeAnalysis::ANALYSIS_UP_TO_DATE) {
        return Ok(());
    }
    if analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) == false { 
        return Err(Error::from_str(&format!("cannot continue with merge, fastforward not possible, analysis is {:?}", analysis)));
    }

    let mut master = repo.find_branch("master", git2::BranchType::Local)?.into_reference();
    master.set_target(oid, "Update local branch master to match remote master")?;
    // HEAD and master now point to the correct location and the files are in the correct
    // state, however all the changes are also in the index
    // HACK if we clear the index then everything seems to be correct so lets just do that
    repo.reset(&repo.revparse_single("HEAD")?, git2::ResetType::Hard, None)?;

    Ok(())
}

pub struct SSHConf<'a> {
    pub username: Option<&'a str>,
    pub public_key: &'a Path,
    pub private_key: &'a Path,
    pub passphrase: Option<&'a str>,
}

fn get_creds(_url: &str, username: Option<&str>, _cred_type: CredentialType, ssh: &SSHConf)
    -> Result<Cred, Error> {

    // a provided username should override the configured username
    let username = match (username, ssh.username) {
        (Some(name), _) => name,
        (None, Some(name)) => name,
        (None, None) => return Err(Error::from_str("username could not be found")),
    };


    // TODO add other type of credential handling in the future
    Cred::ssh_key(
        username,
        Some(ssh.public_key),
        ssh.private_key,
        ssh.passphrase)
}
