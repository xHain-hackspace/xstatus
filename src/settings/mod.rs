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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempConfig(PathBuf);

    impl TempConfig {
        fn new(name: &str, contents: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("xstatus-{}-{}.toml", name, std::process::id()));
            std::fs::write(&path, contents).unwrap();
            TempConfig(path)
        }

        fn path(&self) -> &str {
            self.0.to_str().unwrap()
        }
    }

    impl Drop for TempConfig {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn loads_defaults_without_config_file() {
        let settings = Settings::new("does-not-exist.toml").unwrap();

        assert_eq!(settings.endpoint, "127.0.0.1:8080");
        assert_eq!(settings.path_prefix, "spaceapi");
        assert_eq!(settings.status.space, "xHain hack+makespace");
        assert_eq!(settings.status.state, None);
        assert_eq!(settings.get_api_version().unwrap(), "14");
    }

    #[test]
    fn config_file_overrides_defaults() {
        let config = TempConfig::new(
            "override",
            r#"
endpoint = "0.0.0.0:9000"

[status]
space = "Other Space"
"#,
        );

        let settings = Settings::new(config.path()).unwrap();

        assert_eq!(settings.endpoint, "0.0.0.0:9000");
        assert_eq!(settings.status.space, "Other Space");
        // values not set in the file keep their defaults
        assert_eq!(settings.path_prefix, "spaceapi");
        assert_eq!(settings.status.url, "https://x-hain.de");
    }

    #[test]
    fn invalid_config_file_is_an_error() {
        let config = TempConfig::new("invalid", "endpoint = [");

        assert!(Settings::new(config.path()).is_err());
    }

    #[test]
    fn api_version_missing() {
        let mut settings = Settings::new("does-not-exist.toml").unwrap();

        settings.status.api_compatibility = Some(vec![]);
        assert!(settings.get_api_version().is_err());

        settings.status.api_compatibility = None;
        assert!(settings.get_api_version().is_err());
    }
}
