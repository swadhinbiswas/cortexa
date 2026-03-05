use thiserror::Error;

#[derive(Debug, Error)]
pub enum GCCError {
    #[error("GCC workspace already exists at {path}")]
    AlreadyExists { path: String },

    #[error("GCC workspace not found at {path}")]
    NotFound { path: String },

    #[error("Branch '{name}' already exists")]
    BranchExists { name: String },

    #[error("Branch '{name}' not found")]
    BranchNotFound { name: String },

    #[error("Failed to acquire lock: {0}")]
    Lock(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, GCCError>;
