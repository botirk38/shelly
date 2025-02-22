use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::process::exit;

#[derive(Debug)]
struct Shell {
    current_dir: String,
    history: Vec<String>,
    builtin_commands: HashSet<String>,
}

impl Shell {
    fn new() -> Self {
        env_logger::init();
        info!("Initializing shell");

        let mut builtin_commands = HashSet::new();
        builtin_commands.insert("exit".to_string());
        builtin_commands.insert("history".to_string());
        builtin_commands.insert("echo".to_string());
        builtin_commands.insert("pwd".to_string());
        builtin_commands.insert("type".to_string());

        debug!("Registered builtin commands: {:?}", builtin_commands);

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
        info!("Starting shell REPL");
        loop {
            if !self.prompt() {
                break;
            }
        }
        info!("Shell terminated");
    }

    fn prompt(&mut self) -> bool {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            error!("Failed to read input");
            return false;
        }

        let input = input.trim();
        if input.is_empty() {
            return true;
        }

        debug!("Received command: {}", input);
        self.history.push(input.to_string());
        self.execute_command(input)
    }

    fn execute_command(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if let Some(command) = parts.first() {
            let args = &parts[1..];
            debug!("Executing command: {} with args: {:?}", command, args);

            match self.run_builtin_command(command, args) {
                Ok(should_continue) => should_continue,
                Err(_) => {
                    warn!("Command not found: {}", command);
                    println!("{}: command not found", command);
                    true
                }
            }
        } else {
            true
        }
    }

    fn is_builtin(&self, cmd: &str) -> bool {
        self.builtin_commands.contains(cmd)
    }

    fn run_builtin_command(&mut self, command: &str, args: &[&str]) -> Result<bool, &'static str> {
        debug!("Running builtin command: {} with args: {:?}", command, args);

        match command {
            "exit" => {
                let status = args
                    .first()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                info!("Exiting with status code: {}", status);
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
            "type" => {
                if let Some(cmd) = args.first() {
                    if self.is_builtin(cmd) {
                        println!("{} is a shell builtin", cmd);
                    } else {
                        println!("{}: not found", cmd);
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

