use serde::{Serialize, Deserialize};
use std::io;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    pub mysql: Mysql,
    pub oauth2: Oauth2,
    pub login: Login,
    pub led: Led,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Led {
    pub length: u16,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Mysql {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Oauth2 {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[cfg(not(debug_assertions))]
const PATH: &str = "/etc/deskled/config.toml";
#[cfg(debug_assertions)]
const PATH: &str = "./config.toml";

impl Config {
    pub async fn get() -> io::Result<Self> {
        let path = Path::new(PATH);
        if !path.exists() {
            debug!("Config does not exist, creating default at {path:?}");
            Self::create_default(path).await
        } else {
            debug!("Config exists, opening");
            Self::open(path).await
        }
    }

    async fn create_default(path: &Path) -> io::Result<Self> {
        let this = Self::default();
        let ser = toml::to_string_pretty(&this).expect("Serializing config");

        let mut f = fs::File::create(path).await?;
        f.write_all(ser.as_bytes()).await?;

        Ok(this)
    }

    async fn open(path: &Path) -> io::Result<Self> {
        let mut f = fs::File::open(path).await?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).await?;

        let de: Self = toml::from_slice(&buf).expect("Deserializing TOML");
        Ok(de)
    }
}