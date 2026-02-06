use serde::{Deserialize, Deserializer, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use duration_str::deserialize_duration;
use anyhow::Context;
use log::info;

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
    #[serde(deserialize_with = "deserialize_path", default = "default_key_path")]
    pub key_path: PathBuf,
    #[serde(default = "default_key_validity")]
    pub key_validity: String,
    #[serde(default = "default_pkce_client_id")]
    pub pkce_client_id: String,
    #[serde(default = "default_issuer_url")]
    pub issuer_url: String,
    #[serde(default = "default_keys_url")]
    pub keys_url: String,
    #[serde(default = "default_sign_url")]
    pub sign_url: String,
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

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Self::default();

        let proj_dirs = ProjectDirs::from("ch", "cscs", "cscs-key")
            .context("Could not determine configuration directory")?;
        let config_dir = proj_dirs.config_dir();
        let config_file_path = config_dir.join("config.toml");

        if !config_file_path.exists() {
            info!("Creating default configuration at {:?}", config_file_path);

            fs::create_dir_all(config_dir)
                .with_context(|| format!("Failed to create config directory {:?}", config_dir))?;
            let default_config = Self::default();
            let default_toml = toml::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            fs::write(&config_file_path, default_toml)
                .with_context(|| format!("Failed to write default config file to {:?}", config_file_path))?;

            return Ok(default_config)
        }

        info!("Loading configuration from {:?}", config_file_path);

        let config_str = fs::read_to_string(&config_file_path)
            .with_context(|| format!("Failed to read config file at {:?}", config_file_path))?;
        let config: Config = toml::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file at {:?}", config_file_path))?;

        Ok(config)
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
