use crate::builtin::BuiltinRegistry;
use crate::command::{CommandParser, CommandParts};
use crate::completion::RustylineHelper;
use crate::error::ShellError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

pub struct Shell {
    current_dir: PathBuf,
    builtin_registry: BuiltinRegistry,
    editor: Editor<RustylineHelper, FileHistory>,
}

impl Shell {
    pub fn new() -> Result<Self, ShellError> {
        let current_dir = std::env::current_dir().map_err(ShellError::IoError)?;
        let builtin_registry = BuiltinRegistry::default();
        let builtins: HashSet<String> = builtin_registry
            .get_command_names()
            .into_iter()
            .map(String::from)
            .collect();

        let helper = RustylineHelper::new(builtins);
        let mut editor = Editor::new().map_err(|e| ShellError::EditorError(e.to_string()))?;
        editor.set_helper(Some(helper));
        let _ = editor.load_history("history.txt");

        Ok(Self {
            current_dir,
            builtin_registry,
            editor,
        })
    }

    pub fn run(&mut self) -> Result<(), ShellError> {
        loop {
            let prompt = "$ ";
            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let _ = self.editor.add_history_entry(line);
                    let cmd = CommandParser::parse(line);
                    match self.execute_command(cmd) {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                    let _ = self.editor.save_history("history.txt");
                }
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

    fn execute_builtin(&mut self, cmd: &CommandParts) -> Result<String, ShellError> {
        if let Some(builtin) = self.builtin_registry.get_command(&cmd.command) {
            let result = builtin.execute(&cmd.args, &self.current_dir)?;

            if cmd.command == "cd" {
                self.current_dir = std::env::current_dir().unwrap_or(self.current_dir.clone());
            }

            match (&cmd.output_redirect, &cmd.error_redirect) {
                (Some((path, append)), _) => {
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
                    let _ = std::fs::File::create(path);
                    Ok(result)
                }
                _ => Ok(result),
            }
        } else {
            Ok(String::new())
        }
    }

    fn execute_external(&self, cmd: &CommandParts) -> Result<String, ShellError> {
        let mut process = std::process::Command::new(&cmd.command);
        process.args(&cmd.args).current_dir(&self.current_dir);

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

    fn execute_command(&mut self, cmd: CommandParts) -> Result<String, ShellError> {
        if cmd.command.is_empty() {
            return Ok(String::new());
        }

        if self.builtin_registry.is_builtin(&cmd.command) {
            self.execute_builtin(&cmd)
        } else {
            self.execute_external(&cmd)
        }
    }
}

