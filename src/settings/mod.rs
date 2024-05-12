use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use spaceapi::{ApiVersion, Status};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub endpoint: String,
    pub status: Status,
    pub path_prefix: String,
}

impl Settings {
    pub fn new(path_to_config_file: &str) -> Result<Self, ConfigError> {
        // check if config file exists
        let mut builder = Config::builder()
            .add_source(File::from_str(
                include_str!("default_config.toml"),
                FileFormat::Toml,
            ))
            .add_source(Environment::with_prefix("XSTATUS"));

        if std::fs::metadata(path_to_config_file).is_ok() {
            builder = builder.add_source(File::with_name(path_to_config_file));
        }
        builder.build()?.try_deserialize()
    }

    pub fn get_api_version(&self) -> Result<String, ConfigError> {
        match self
            .status
            .api_compatibility
            .as_ref()
            .ok_or(ConfigError::NotFound("api compatibility".to_string()))?
            .first()
            .ok_or(ConfigError::NotFound("api compatibility".to_string()))?
        {
            ApiVersion::V14 => Ok("14".to_string()),
        }
    }
}
