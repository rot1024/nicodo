use serde::{Deserialize, Serialize};

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub session: String,
}

impl Config {
    pub fn load() -> Result<Self, confy::ConfyError> {
        confy::load::<Config>(APP_NAME)
    }

    pub fn save(&self) -> Result<(), confy::ConfyError> {
        confy::store(APP_NAME, &self)
    }
}
