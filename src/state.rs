use std::path::PathBuf;
use directories::ProjectDirs;
use std::fs;
use serde::{Serialize, Deserialize};
use serde_json;
use chrono::{DateTime, Utc, Duration};
use log::info;

#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    pub oidc_token: Option<TokenStore>,
    pub ssh_cert: Option<CertMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenStore {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expiration: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct CertMetadata {
    pub key_path: PathBuf,
    pub cert_path: PathBuf,
    pub serial_number: String,
    pub expires_at: String,
}

impl AppState {
    fn get_path() -> anyhow::Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("ch", "cscs", "cscs-key")
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let cache_dir = proj_dirs.cache_dir();
        fs::create_dir_all(cache_dir)?;
        Ok(cache_dir.join("token.json"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::get_path()?;
        info!("Trying to load state from {}", path.display());
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_path()?;
        info!("Saving state to {}", path.display());
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

impl TokenStore {
    pub fn is_expired(&self) -> bool {
        let grace_period = Duration::seconds(10);
        match self.expiration {
            Some(expire_at) => Utc::now() + grace_period > expire_at,
            None => true,
        }
    }
}
