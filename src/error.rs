use std::fmt;
use std::io;

/// Error types for shell operations
///
/// Provides structured error handling for various shell failures
/// including I/O errors, command execution errors, and environment issues.
#[derive(Debug)]
pub enum ShellError {
    /// I/O operation failed
    IoError(io::Error),
    /// Command not found in PATH or built-ins
    CommandNotFound(String),
    /// External command execution failed
    ExecutionError(String),
    /// Rustyline editor error
    EditorError(String),
    /// Environment variable not found
    EnvVarNotFound(String),
    /// Directory not found
    DirectoryNotFound(String),
    /// Change directory failed (path, error message)
    CdError(String, String),
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

/// Auto-convert io::Error to ShellError for convenience
impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::IoError(err)
    }
}

