
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub site_root: PathBuf,
    pub ssh: SSHConf,
}

#[derive(Debug)]
pub struct SSHConf {
    pub username: Option<String>,
    pub passphrase: Option<String>,
    pub public_key: PathBuf,
    pub private_key: PathBuf,
}

