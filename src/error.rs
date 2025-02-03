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

    #[error("Conflict with existing configuration: {0}")]
    ConflictError(String),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("File read error: {0}")]
    FileReadError(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::FileWriteError(error.to_string())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::FileWriteError(error.to_string())
    }
}
