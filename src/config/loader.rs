use crate::embedder::EmbedderConfig;

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub embedder: EmbedderConfig,
}

#[derive(Debug)]
pub enum ConfigError {
    UnableToLoadFromFlle(String),
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::UnableToLoadFromFlle(e.to_string()))?;

        toml::from_str(&content).map_err(|e| ConfigError::UnableToLoadFromFlle(e.to_string()))
    }
}
