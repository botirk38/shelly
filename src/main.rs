use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{exit, Command};

#[derive(Debug)]
struct Shell {
    current_dir: PathBuf,
    history: Vec<String>,
    builtin_commands: HashSet<String>,
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

    fn find_executable(&self, cmd: &str) -> Option<PathBuf> {
        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(cmd);
                full_path.exists().then_some(full_path)
            })
        })
    }

    fn parse_command(&self, input: &str) -> Vec<String> {
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
        let tokens = self.parse_command(input);

        if let Some(command) = tokens.first() {
            let args = &tokens[1..];

            if self.is_builtin(command) {
                return self.run_builtin_command(command, args).unwrap_or(true);
            }

            if self.find_executable(command).is_some() {
                match Command::new(command)
                    .args(args)
                    .current_dir(&self.current_dir)
                    .spawn()
                {
                    Ok(mut child) => {
                        let _ = child.wait();
                        return true;
                    }
                    Err(_) => {
                        println!("{}: command not found", command);
                        return true;
                    }
                }
            }
            println!("{}: command not found", command);
        }
        true
    }

    fn is_builtin(&self, cmd: &str) -> bool {
        self.builtin_commands.contains(cmd)
    }

    fn run_builtin_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<bool, &'static str> {
        match command {
            "exit" => {
                let status = args
                    .first()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                exit(status);
            }
            "history" => {
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("{}: {}", i + 1, cmd);
                }
                Ok(true)
            }
            "echo" => {
                if !args.is_empty() {
                    println!("{}", args.join(" "));
                }
                Ok(true)
            }
            "pwd" => {
                println!("{}", self.current_dir.display());
                Ok(true)
            }
            "cd" => {
                match args.first() {
                    Some(dir) if dir == "~" => self.change_to_home_dir(),
                    Some(dir) if dir.starts_with("~/") => {
                        let home = env::var("HOME").unwrap_or_default();
                        self.change_directory(&format!("{}{}", home, &dir[1..]))
                    }
                    Some(dir) => self.change_directory(dir),
                    None => self.change_to_home_dir(),
                }
                Ok(true)
            }
            "type" => {
                if let Some(cmd) = args.first() {
                    if self.is_builtin(cmd) {
                        println!("{} is a shell builtin", cmd);
                    } else if let Some(path) = self.find_executable(cmd) {
                        println!("{} is {}", cmd, path.display());
                    } else {
                        println!("{}: not found", cmd);
                    }
                }
                Ok(true)
            }
            _ => Err("Command not found"),
        }
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

