use crate::builtin::BuiltinRegistry;
use crate::command::{CommandParser, CommandParts};
use crate::completion::RustylineHelper;
use crate::error::ShellError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

/// The main shell structure that manages command execution and interactive input
pub struct Shell {
    /// Current working directory
    current_dir: PathBuf,
    /// Registry of built-in commands
    builtin_registry: BuiltinRegistry,
    /// Rustyline editor with history and completion support
    editor: Editor<RustylineHelper, FileHistory>,
}

impl Shell {
    /// Create a new Shell instance
    ///
    /// Initializes the shell with:
    /// - Current working directory
    /// - Built-in command registry
    /// - Rustyline editor with tab completion and history
    pub fn new() -> Result<Self, ShellError> {
        let current_dir = std::env::current_dir().map_err(ShellError::IoError)?;
        let builtin_registry = BuiltinRegistry::default();

        // Collect built-in command names for tab completion
        let builtins: HashSet<String> = builtin_registry
            .get_command_names()
            .into_iter()
            .map(String::from)
            .collect();

        // Set up editor with completion helper
        let helper = RustylineHelper::new(builtins);
        let mut editor = Editor::new().map_err(|e| ShellError::EditorError(e.to_string()))?;
        editor.set_helper(Some(helper));

        // Load command history from file (ignore errors if file doesn't exist)
        let _ = editor.load_history("history.txt");

        Ok(Self {
            current_dir,
            builtin_registry,
            editor,
        })
    }

    /// Main REPL (Read-Eval-Print Loop) for the shell
    ///
    /// Continuously reads user input, parses and executes commands,
    /// and displays output until interrupted or EOF.
    pub fn run(&mut self) -> Result<(), ShellError> {
        loop {
            let prompt = "$ ";
            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Parse and execute command
                    let cmd = CommandParser::parse(line);
                    match self.execute_command(cmd) {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }

                    // Save history after each command
                    let _ = self.editor.save_history("history.txt");
                }
                // Handle Ctrl+C or Ctrl+D
                Err(rustyline::error::ReadlineError::Interrupted)
                | Err(rustyline::error::ReadlineError::Eof) => {
                    break;
                }
                Err(e) => {
                    return Err(ShellError::EditorError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Execute a built-in command with output/error redirection support
    fn execute_builtin(&mut self, cmd: &CommandParts) -> Result<String, ShellError> {
        if let Some(builtin) = self.builtin_registry.get_command(&cmd.command) {
            let result = builtin.execute(&cmd.args, &self.current_dir)?;

            // Update current_dir after cd command
            if cmd.command == "cd" {
                self.current_dir = std::env::current_dir().unwrap_or(self.current_dir.clone());
            }

            // Handle output/error redirection
            match (&cmd.output_redirect, &cmd.error_redirect) {
                (Some((path, append)), _) => {
                    // Redirect stdout to file
                    let mut file = if *append {
                        std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(path)?
                    } else {
                        std::fs::File::create(path)?
                    };
                    writeln!(file, "{}", result)?;
                    Ok(String::new())
                }
                (_, Some((path, _))) => {
                    // Create error redirect file (built-ins don't typically write to stderr)
                    let _ = std::fs::File::create(path);
                    Ok(result)
                }
                _ => Ok(result),
            }
        } else {
            Ok(String::new())
        }
    }

    /// Execute an external command (not a built-in)
    ///
    /// Spawns a child process and waits for it to complete.
    /// Handles stdout and stderr redirection if specified.
    fn execute_external(&self, cmd: &CommandParts) -> Result<String, ShellError> {
        let mut process = std::process::Command::new(&cmd.command);
        process.args(&cmd.args).current_dir(&self.current_dir);

        // Set up stdout redirection if specified
        if let Some((path, append)) = &cmd.output_redirect {
            let file = if *append {
                std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)?
            } else {
                std::fs::File::create(path)?
            };
            process.stdout(file);
        }

        // Set up stderr redirection if specified
        if let Some((path, append)) = &cmd.error_redirect {
            let file = if *append {
                std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)?
            } else {
                std::fs::File::create(path)?
            };
            process.stderr(file);
        }

        // Spawn process and wait for completion
        match process.spawn() {
            Ok(mut child) => {
                child
                    .wait()
                    .map_err(|e| ShellError::ExecutionError(e.to_string()))?;
                Ok(String::new())
            }
            Err(_) => {
                println!("{}: command not found", cmd.command);
                Ok(String::new())
            }
        }
    }

    /// Execute a command, dispatching to either built-in or external execution
    ///
    /// Built-in commands are checked first for efficiency.
    fn execute_command(&mut self, cmd: CommandParts) -> Result<String, ShellError> {
        if cmd.command.is_empty() {
            return Ok(String::new());
        }

        // Check if it's a built-in command first
        if self.builtin_registry.is_builtin(&cmd.command) {
            self.execute_builtin(&cmd)
        } else {
            self.execute_external(&cmd)
        }
    }
}

