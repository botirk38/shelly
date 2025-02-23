use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ShellError {
    IoError(io::Error),
    CommandNotFound(String),
    ExecutionError(String),
    EditorError(String),
    EnvVarNotFound(String),
    DirectoryNotFound(String),
    CdError(String, String), // (path, error message)
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::IoError(err) => write!(f, "IO error: {}", err),
            ShellError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            ShellError::ExecutionError(err) => write!(f, "Execution error: {}", err),
            ShellError::EditorError(err) => write!(f, "Editor error: {}", err),
            ShellError::EnvVarNotFound(var) => write!(f, "Environment variable not found: {}", var),
            ShellError::DirectoryNotFound(dir) => write!(f, "Directory not found: {}", dir),
            ShellError::CdError(path, msg) => write!(f, "cd: {}: {}", path, msg),
        }
    }
}

impl std::error::Error for ShellError {}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::IoError(err)
    }
}

