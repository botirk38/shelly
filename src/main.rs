use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{exit, Command};

#[derive(Debug)]
struct Shell {
    current_dir: String,
    history: Vec<String>,
    builtin_commands: HashSet<String>,
}

impl Shell {
    fn new() -> Self {
        let mut builtin_commands = HashSet::new();
        builtin_commands.insert("exit".to_string());
        builtin_commands.insert("history".to_string());
        builtin_commands.insert("echo".to_string());
        builtin_commands.insert("pwd".to_string());
        builtin_commands.insert("type".to_string());
        builtin_commands.insert("cd".to_string());

        Shell {
            current_dir: env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            history: Vec::new(),
            builtin_commands,
        }
    }

    fn run(&mut self) {
        loop {
            if !self.prompt() {
                break;
            }
        }
    }

    fn prompt(&mut self) -> bool {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
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

    fn find_executable(&self, cmd: &str) -> Option<String> {
        if let Ok(path) = env::var("PATH") {
            for dir in path.split(':') {
                let full_path = Path::new(dir).join(cmd);
                if full_path.exists() {
                    return Some(full_path.to_string_lossy().into_owned());
                }
            }
        }
        None
    }

    fn execute_command(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if let Some(command) = parts.first() {
            let args = &parts[1..];

            if self.is_builtin(command) {
                return self.run_builtin_command(command, args).unwrap_or(true);
            }

            if self.find_executable(command).is_some() {
                match Command::new(command).args(args).spawn() {
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

    fn run_builtin_command(&mut self, command: &str, args: &[&str]) -> Result<bool, &'static str> {
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
                println!("{}", args.join(" "));
                Ok(true)
            }
            "pwd" => {
                println!("{}", self.current_dir);
                Ok(true)
            }

            "cd" => {
                if let Some(dir) = args.first() {
                    match env::set_current_dir(dir) {
                        Ok(_) => {
                            self.current_dir = env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                        }
                        Err(_) => {
                            println!("cd: {}: No such file or directory", dir);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(true)
                }
            }
            "type" => {
                if let Some(cmd) = args.first() {
                    if self.is_builtin(cmd) {
                        println!("{} is a shell builtin", cmd);
                    } else {
                        match self.find_executable(cmd) {
                            Some(path) => println!("{} is {}", cmd, path),
                            None => println!("{}: not found", cmd),
                        }
                    }
                }
                Ok(true)
            }
            _ => Err("Command not found"),
        }
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}

