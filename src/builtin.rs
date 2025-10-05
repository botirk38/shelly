use crate::error::ShellError;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Trait for implementing built-in shell commands
///
/// Each built-in command implements this trait to provide its name
/// and execution logic. Commands receive arguments and the current
/// working directory, and return output as a String.
pub trait BuiltinCommand {
    /// Return the command name (e.g., "cd", "echo")
    fn name(&self) -> &'static str;

    /// Execute the command with given arguments
    ///
    /// # Arguments
    /// * `args` - Command arguments (not including the command name itself)
    /// * `working_dir` - Current working directory
    ///
    /// # Returns
    /// Command output as a String, or an error
    fn execute(&self, args: &[String], working_dir: &Path) -> Result<String, ShellError>;
}

/// Registry that holds all built-in commands
///
/// Uses a HashMap for O(1) command lookup by name.
pub struct BuiltinRegistry {
    commands: HashMap<String, Box<dyn BuiltinCommand>>,
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinRegistry {
    /// Create a new registry with all built-in commands registered
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        // Register all built-in commands
        registry.register(Box::new(CdCommand));
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(PwdCommand));
        registry.register(Box::new(ExitCommand));
        registry.register(Box::new(TypeCommand));
        registry.register(Box::new(HistoryCommand));
        registry
    }

    /// Get a list of all registered command names
    pub fn get_command_names(&self) -> Vec<String> {
        self.commands
            .values()
            .map(|cmd| cmd.name().to_string())
            .collect()
    }

    /// Register a new built-in command
    pub fn register(&mut self, command: Box<dyn BuiltinCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }

    /// Get a command by name
    pub fn get_command(&self, name: &str) -> Option<&dyn BuiltinCommand> {
        self.commands.get(name).map(|cmd| cmd.as_ref())
    }

    /// Check if a command name is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
}

/// Change directory command
struct CdCommand;

impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        // Determine target directory: HOME if no args, otherwise the specified path
        // Handles ~ and ~/ expansion
        let target_dir = match args.first() {
            Some(dir) if dir == "~" => {
                env::var("HOME").map_err(|_| ShellError::EnvVarNotFound("HOME".to_string()))?
            }
            Some(dir) if dir.starts_with("~/") => {
                let home =
                    env::var("HOME").map_err(|_| ShellError::EnvVarNotFound("HOME".to_string()))?;
                format!("{}{}", home, &dir[1..])
            }
            Some(dir) => dir.clone(),
            None => env::var("HOME").map_err(|_| ShellError::EnvVarNotFound("HOME".to_string()))?,
        };

        // Attempt to change directory
        if env::set_current_dir(&target_dir).is_err() {
            return Ok(format!("cd: {}: No such file or directory", target_dir));
        }
        Ok(String::new())
    }
}

/// Print arguments to stdout
struct EchoCommand;

impl BuiltinCommand for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        Ok(args.join(" "))
    }
}

/// Print working directory command
struct PwdCommand;

impl BuiltinCommand for PwdCommand {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn execute(&self, _args: &[String], working_dir: &Path) -> Result<String, ShellError> {
        Ok(working_dir.display().to_string())
    }
}

/// Exit the shell with optional status code
struct ExitCommand;

impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        // Parse exit code from first argument, default to 0
        let status = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        std::process::exit(status);
    }
}

/// Determine the type of a command (builtin or executable path)
struct TypeCommand;

impl BuiltinCommand for TypeCommand {
    fn name(&self) -> &'static str {
        "type"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        if let Some(cmd) = args.first() {
            // Check if it's a built-in command
            if BUILTIN_COMMANDS.contains(&cmd.as_str()) {
                return Ok(format!("{} is a shell builtin", cmd));
            }
            // Check if it's an executable in PATH
            if let Some(path) = find_executable(cmd) {
                return Ok(format!("{} is {}", cmd, path.display()));
            }
            return Ok(format!("{}: not found", cmd));
        }
        Ok(String::new())
    }
}

/// Display command history (currently not implemented)
struct HistoryCommand;

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }

    fn execute(&self, _args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        // History is managed by rustyline, not implemented here
        Ok(String::new())
    }
}

/// Search for an executable in PATH
fn find_executable(cmd: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let full_path = dir.join(cmd);
            full_path.exists().then_some(full_path)
        })
    })
}

/// List of all built-in command names
const BUILTIN_COMMANDS: &[&str] = &["cd", "echo", "pwd", "exit", "type", "history"];
