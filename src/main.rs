use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug)]
struct Shell {
    current_dir: PathBuf,
    history: Vec<String>,
    builtin_commands: HashSet<String>,
}

#[derive(Debug)]
struct CommandParts {
    command: String,
    args: Vec<String>,
    output_redirect: Option<(String, bool)>,
    error_redirect: Option<(String, bool)>,
}

#[derive(PartialEq)]
enum QuoteState {
    None,
    Single,
    Double,
}

impl Shell {
    fn new() -> Self {
        let builtin_commands: HashSet<String> =
            vec!["exit", "history", "echo", "pwd", "type", "cd"]
                .into_iter()
                .map(String::from)
                .collect();

        Shell {
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            history: Vec::with_capacity(100),
            builtin_commands,
        }
    }

    fn run(&mut self) {
        while self.prompt() {}
    }

    fn prompt(&mut self) -> bool {
        print!("$ ");
        if io::stdout().flush().is_err() {
            return false;
        }

        let mut input = String::with_capacity(64);
        if io::stdin().read_line(&mut input).is_err() {
            return false;
        }

        let input = input.trim();
        if input.is_empty() {
            return true;
        }

        self.history.push(input.to_string());
        self.execute_command(input)
    }

    fn parse_command(&self, input: &str) -> CommandParts {
        let tokens = self.tokenize(input);
        let mut command_parts = CommandParts {
            command: String::new(),
            args: Vec::new(),
            output_redirect: None,
            error_redirect: None,
        };

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i].as_str() {
                ">" | "1>" if i + 1 < tokens.len() => {
                    command_parts.output_redirect = Some((tokens[i + 1].clone(), false));
                    i += 2;
                }
                ">>" | "1>>" if i + 1 < tokens.len() => {
                    command_parts.output_redirect = Some((tokens[i + 1].clone(), true));
                    i += 2;
                }
                "2>" if i + 1 < tokens.len() => {
                    command_parts.error_redirect = Some((tokens[i + 1].clone(), false));
                    i += 2;
                }
                "2>>" if i + 1 < tokens.len() => {
                    command_parts.error_redirect = Some((tokens[i + 1].clone(), true));
                    i += 2;
                }
                token => {
                    if command_parts.command.is_empty() {
                        command_parts.command = token.to_string();
                    } else {
                        command_parts.args.push(token.to_string());
                    }
                    i += 1;
                }
            }
        }
        command_parts
    }

    fn tokenize(&self, input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut quote_state = QuoteState::None;
        let mut chars = input.chars();

        while let Some(c) = chars.next() {
            match c {
                '\'' if quote_state != QuoteState::Double => {
                    quote_state = if quote_state == QuoteState::Single {
                        QuoteState::None
                    } else {
                        QuoteState::Single
                    };
                }
                '"' if quote_state != QuoteState::Single => {
                    quote_state = if quote_state == QuoteState::Double {
                        QuoteState::None
                    } else {
                        QuoteState::Double
                    };
                }
                '\\' => {
                    if let Some(next_char) = chars.next() {
                        match quote_state {
                            QuoteState::Double => {
                                if next_char == '"' || next_char == '\\' {
                                    current_token.push(next_char);
                                } else {
                                    current_token.push('\\');
                                    current_token.push(next_char);
                                }
                            }
                            QuoteState::None => current_token.push(next_char),
                            QuoteState::Single => {
                                current_token.push('\\');
                                current_token.push(next_char);
                            }
                        }
                    }
                }
                ' ' if quote_state == QuoteState::None => {
                    if !current_token.is_empty() {
                        tokens.push(current_token);
                        current_token = String::new();
                    }
                }
                _ => current_token.push(c),
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    fn execute_command(&mut self, input: &str) -> bool {
        let cmd_parts = self.parse_command(input);
        if cmd_parts.command.is_empty() {
            return true;
        }

        if self.is_builtin(&cmd_parts.command) {
            self.execute_builtin(&cmd_parts)
        } else {
            self.execute_external(&cmd_parts)
        }
    }

    fn execute_builtin(&mut self, cmd_parts: &CommandParts) -> bool {
        match cmd_parts.command.as_str() {
            "exit" => {
                let status = cmd_parts
                    .args
                    .first()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                std::process::exit(status);
            }
            "echo" => {
                let output = cmd_parts.args.join(" ");
                if let Some((path, append)) = &cmd_parts.output_redirect {
                    let file = if *append {
                        File::options().append(true).create(true).open(path)
                    } else {
                        File::create(path)
                    };
                    if let Ok(mut file) = file {
                        let _ = writeln!(file, "{}", output);
                    }
                } else if let Some((path, _)) = &cmd_parts.error_redirect {
                    // For echo, we don't write to stderr redirect file
                    // Just create the file if it doesn't exist
                    let _ = File::create(path);
                    println!("{}", output);
                } else {
                    println!("{}", output);
                }
            }
            "pwd" => {
                let output = self.current_dir.display().to_string();
                if let Some((path, append)) = &cmd_parts.error_redirect {
                    let file = if *append {
                        File::options().append(true).create(true).open(path)
                    } else {
                        File::create(path)
                    };
                    if let Ok(mut file) = file {
                        let _ = writeln!(file, "{}", output);
                    }
                } else {
                    println!("{}", output);
                }
            }
            "history" => {
                let output = self
                    .history
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| format!("{}: {}", i + 1, cmd))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some((path, append)) = &cmd_parts.error_redirect {
                    let file = if *append {
                        File::options().append(true).create(true).open(path)
                    } else {
                        File::create(path)
                    };
                    if let Ok(mut file) = file {
                        let _ = writeln!(file, "{}", output);
                    }
                } else {
                    println!("{}", output);
                }
            }
            "cd" => match cmd_parts.args.first() {
                Some(dir) if dir == "~" => self.change_to_home_dir(),
                Some(dir) if dir.starts_with("~/") => {
                    if let Ok(home) = env::var("HOME") {
                        let path = format!("{}{}", home, &dir[1..]);
                        self.change_directory(&path);
                    }
                }
                Some(dir) => self.change_directory(dir),
                None => self.change_to_home_dir(),
            },
            "type" => {
                let output = if let Some(cmd) = cmd_parts.args.first() {
                    if self.is_builtin(cmd) {
                        format!("{} is a shell builtin", cmd)
                    } else if let Some(path) = self.find_executable(cmd) {
                        format!("{} is {}", cmd, path.display())
                    } else {
                        format!("{}: not found", cmd)
                    }
                } else {
                    String::new()
                };
                if let Some((path, append)) = &cmd_parts.error_redirect {
                    let file = if *append {
                        File::options().append(true).create(true).open(path)
                    } else {
                        File::create(path)
                    };
                    if let Ok(mut file) = file {
                        let _ = writeln!(file, "{}", output);
                    }
                } else {
                    println!("{}", output);
                }
            }
            _ => {}
        }
        true
    }

    fn execute_external(&self, cmd_parts: &CommandParts) -> bool {
        if self.find_executable(&cmd_parts.command).is_none() {
            println!("{}: command not found", cmd_parts.command);
            return true;
        }

        let mut command = Command::new(&cmd_parts.command);
        command.args(&cmd_parts.args).current_dir(&self.current_dir);

        if let Some((path, append)) = &cmd_parts.output_redirect {
            let file = if *append {
                File::options().append(true).create(true).open(path)
            } else {
                File::create(path)
            };
            if let Ok(file) = file {
                command.stdout(Stdio::from(file));
            }
        }

        if let Some((path, append)) = &cmd_parts.error_redirect {
            let file = if *append {
                File::options().append(true).create(true).open(path)
            } else {
                File::create(path)
            };
            if let Ok(file) = file {
                command.stderr(Stdio::from(file));
            }
        }

        match command.spawn() {
            Ok(mut child) => {
                let _ = child.wait();
                true
            }
            Err(_) => {
                println!("{}: command not found", cmd_parts.command);
                true
            }
        }
    }

    fn find_executable(&self, cmd: &str) -> Option<PathBuf> {
        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(cmd);
                full_path.exists().then_some(full_path)
            })
        })
    }

    fn is_builtin(&self, cmd: &str) -> bool {
        self.builtin_commands.contains(cmd)
    }

    fn change_directory(&mut self, path: &str) {
        match env::set_current_dir(path) {
            Ok(_) => {
                self.current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            }
            Err(_) => {
                println!("cd: {}: No such file or directory", path);
            }
        }
    }

    fn change_to_home_dir(&mut self) {
        if let Ok(home) = env::var("HOME") {
            self.change_directory(&home);
        }
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}

