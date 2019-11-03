use super::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::{fs, prelude::*};

const CONFIG_FILENAME: &str = "nicodo_config.json";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
  pub session: String,
}

impl Config {
  pub async fn load() -> Option<Self> {
    tokio::fs::read(CONFIG_FILENAME)
      .await
      .map(|data| {
        serde_json::from_slice::<Self>(&data)
          .map(|c| Some(c))
          .unwrap_or(None)
      })
      .unwrap_or(None)
  }

  pub async fn save(&self) -> Result<()> {
    let mut file = fs::File::create(CONFIG_FILENAME)
      .await
      .map_err(|err| Error::Config(err))?;
    file
      .write_all(&serde_json::to_vec(self).unwrap())
      .await
      .map_err(|err| Error::Config(err))?;
    Ok(())
  }
}
