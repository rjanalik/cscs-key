use clap::Parser;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

// Default config values
fn default_key_path() -> PathBuf {
    let home_dir = dirs::home_dir()
        //.ok_or_else(|| panic!("Could not determine home directory"))?;
        .unwrap_or_else(|| panic!("Could not determine home directory"));
    let relative_path = PathBuf::from(".ssh/cscs-key");
    home_dir.join(relative_path)
}
fn default_key_validity() -> String { "1min".to_string() }
fn default_pkce_client_id() -> String { "authx-cli".to_string() }
fn default_issuer_url() -> String { "https://auth.cscs.ch/auth/realms/cscs".to_string() }
fn default_keys_url() -> String { "https://api-ssh-service.hpc-ssh.svc.cscs.ch/api/v1/ssh-keys".to_string() }
fn default_sign_url() -> String { "https://api-ssh-service.hpc-ssh.svc.cscs.ch/api/v1/ssh-keys/sign".to_string() }

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub key_path: PathBuf,
    pub key_validity: String,
    pub pkce_client_id: String,
    pub issuer_url: String,
    pub keys_url: String,
    pub sign_url: String,
}

#[derive(Parser, Debug, Deserialize, Serialize)]
pub struct ConfigCliOverride {
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<PathBuf>,
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_validity: Option<String>,
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkce_client_id: Option<String>,
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_url: Option<String>,
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys_url: Option<String>,
    #[arg(long, global = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_path: default_key_path(),
            key_validity: default_key_validity(),
            pkce_client_id: default_pkce_client_id(),
            issuer_url: default_issuer_url(),
            keys_url: default_keys_url(),
            sign_url: default_sign_url(),
        }
    }
}

//Resolve path, e.g. "~"
fn deserialize_path<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path_str = String::deserialize(d)?;

    if path_str.starts_with("~/") {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| serde::de::Error::custom("Could not determine home directory"))?;
            //.unwrap_or_else(|| panic!("Could not determine home directory for path: {}", path_str));

            if path_str == "~" {
                Ok(home_dir)
            } else {
                // Remove "~/" and append to home_dir
                Ok(home_dir.join(&path_str[2..]))
            }
    } else {
        // Does not start wit '~' => Return as is
        Ok(PathBuf::from(path_str))
    }
}
