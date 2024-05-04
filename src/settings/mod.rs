use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use spaceapi::Status;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub endpoint: String,
    pub status: Status,
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
}
