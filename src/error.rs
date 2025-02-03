use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration '{0}' already exists")]
    AlreadyExists(String),

    #[error("Missing prerequisite: {0}")]
    MissingPrerequisite(String),

    #[error("Failed to write configuration file: {0}")]
    FileWriteError(String),

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),

    #[error("Dependency installation failed: {0}")]
    DependencyError(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::FileWriteError(error.to_string())
    }
}
