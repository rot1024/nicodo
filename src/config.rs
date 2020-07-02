use super::error::{Error, Result};
use nicodo::Session;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

const CONFIG_DIR: &str = "nicodo";
const CONFIG_FILENAME: &str = "nicodo_config.json";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub session: String,
}

fn get_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join(CONFIG_DIR))
}

impl Config {
    pub async fn load() -> Option<Self> {
        let mut res = tokio::fs::read(CONFIG_FILENAME).await;

        if let Err(err) = &res {
            if let ErrorKind::NotFound = err.kind() {
                if let Some(d) = get_dir() {
                    res = fs::read(d.join(CONFIG_FILENAME)).await;
                }
            }
        }

        match res {
            Ok(data) => serde_json::from_slice::<Self>(&data)
                .map(|c| Some(c))
                .unwrap_or(None),
            _ => None,
        }
    }

    pub async fn save(&self) -> Result<()> {
        let dir = match get_dir() {
            Some(d) => d,
            None => PathBuf::new(),
        };

        if !dir.as_os_str().is_empty() {
            fs::create_dir_all(&dir).await.ok();
        }

        let mut file = fs::File::create(dir.join(CONFIG_FILENAME))
            .await
            .map_err(|err| Error::Config(err))?;
        file.write_all(&serde_json::to_vec(self).unwrap())
            .await
            .map_err(|err| Error::Config(err))?;
        Ok(())
    }

    pub fn session(&self) -> Session {
        Session::from_cookie(&self.session)
    }
}
