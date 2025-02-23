use crate::error::ShellError;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

pub trait BuiltinCommand {
    fn name(&self) -> &'static str;
    fn execute(&self, args: &[String], working_dir: &Path) -> Result<String, ShellError>;
}

pub struct BuiltinRegistry {
    commands: HashMap<String, Box<dyn BuiltinCommand>>,
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        registry.register(Box::new(CdCommand));
        registry.register(Box::new(EchoCommand));
        registry.register(Box::new(PwdCommand));
        registry.register(Box::new(ExitCommand));
        registry.register(Box::new(TypeCommand));
        registry.register(Box::new(HistoryCommand));
        registry
    }

    pub fn get_command_names(&self) -> Vec<String> {
        self.commands
            .values()
            .map(|cmd| cmd.name().to_string())
            .collect()
    }

    pub fn register(&mut self, command: Box<dyn BuiltinCommand>) {
        self.commands.insert(command.name().to_string(), command);
    }

    pub fn get_command(&self, name: &str) -> Option<&dyn BuiltinCommand> {
        self.commands.get(name).map(|cmd| cmd.as_ref())
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
}

struct CdCommand;

impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
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

        if env::set_current_dir(&target_dir).is_err() {
            return Ok(format!("cd: {}: No such file or directory", target_dir));
        }
        Ok(String::new())
    }
}

struct EchoCommand;

impl BuiltinCommand for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        Ok(args.join(" "))
    }
}

struct PwdCommand;

impl BuiltinCommand for PwdCommand {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn execute(&self, _args: &[String], working_dir: &Path) -> Result<String, ShellError> {
        Ok(working_dir.display().to_string())
    }
}

struct ExitCommand;

impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        let status = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        std::process::exit(status);
    }
}

struct TypeCommand;

impl BuiltinCommand for TypeCommand {
    fn name(&self) -> &'static str {
        "type"
    }

    fn execute(&self, args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        if let Some(cmd) = args.first() {
            if BUILTIN_COMMANDS.contains(&cmd.as_str()) {
                return Ok(format!("{} is a shell builtin", cmd));
            }
            if let Some(path) = find_executable(cmd) {
                return Ok(format!("{} is {}", cmd, path.display()));
            }
            return Ok(format!("{}: not found", cmd));
        }
        Ok(String::new())
    }
}

struct HistoryCommand;

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }

    fn execute(&self, _args: &[String], _working_dir: &Path) -> Result<String, ShellError> {
        Ok(String::new())
    }
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let full_path = dir.join(cmd);
            full_path.exists().then_some(full_path)
        })
    })
}

const BUILTIN_COMMANDS: &[&str] = &["cd", "echo", "pwd", "exit", "type", "history"];
