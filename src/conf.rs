
use std::path::PathBuf;

use rocket;

#[derive(Debug)]
pub struct Config {
    pub site_root: PathBuf,
    pub ssh: SSHConf,
}

#[derive(Debug)]
pub struct SSHConf {
    pub username: Option<String>,
    pub public_key: PathBuf,
    pub private_key: PathBuf,
    pub passphrase: Option<String>,
}

impl Config {
    pub fn from_rocket_conf(conf: &rocket::config::Config) -> rocket::config::Result<Self> {
        let root = conf.get_str("site_root")?;
        let ssh = conf.get_table("sshconf")?;
        let pubk = ssh.get("public_key")
            .map(|v| v.as_str().expect("public_key should be a string"))
            .expect("missing required 'public_key'");
        let privk = ssh.get("private_key")
            .map(|v| v.as_str().expect("private_key should be a string"))
            .expect("missing required 'private_key'");
        let user = ssh.get("username")
            .map(|v| v.as_str().expect("username should be a string").to_string());
        let pass = ssh.get("passphrase")
            .map(|v| v.as_str().expect("passphrase should be a string").to_string());

        Ok(Config {
            site_root: PathBuf::from(root),
            ssh: SSHConf{
                username: user,
                public_key: PathBuf::from(pubk),
                private_key: PathBuf::from(privk),
                passphrase: pass,
            },
        })
    }
}

