use std::error::Error;

#[derive(Debug)]
pub enum ConfigError {
    FileRead(std::io::Error),
    JsonParse(serde_json::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FileRead(e) => write!(f, "Failed to read the configuration file: {}", e),
            ConfigError::JsonParse(e) => write!(f, "Failed to parse JSON in the configuration file: {}", e),
        }
    }
}
impl Error for ConfigError {}