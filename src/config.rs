use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
            key_path: dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".ssh/cscs-key"),
            key_validity: "1min".to_string(),
            pkce_client_id: "authx-cli".to_string(),
            issuer_url: "https://auth.cscs.ch/auth/realms/cscs".to_string(),
            keys_url: "https://api-ssh-service.hpc-ssh.svc.cscs.ch/api/v1/ssh-keys".to_string(),
            sign_url: "https://api-ssh-service.hpc-ssh.svc.cscs.ch/api/v1/ssh-keys/sign".to_string(),
        }
    }
}
