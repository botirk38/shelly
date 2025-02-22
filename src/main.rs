use std::env;
use std::io::{self, Write};
use std::process::{exit, Command};

#[derive(Debug)]
struct Shell {
    current_dir: String,
    history: Vec<String>,
}

impl Shell {
    fn new() -> Self {
        Shell {
            current_dir: env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            history: Vec::new(),
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

    fn execute_command(&mut self, input: &str) -> bool {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if let Some(command) = parts.first() {
            match *command {
                "exit" => {
                    let status = parts
                        .get(1)
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(0);
                    exit(status);
                }
                "history" => {
                    for (i, cmd) in self.history.iter().enumerate() {
                        println!("{}: {}", i + 1, cmd);
                    }
                }
                cmd => {
                    let args = &parts[1..];
                    match Command::new(cmd)
                        .args(args)
                        .current_dir(&self.current_dir)
                        .spawn()
                    {
                        Ok(mut child) => {
                            if child.wait().is_err() {
                                println!("{}: command not found", cmd);
                            }
                        }
                        Err(_) => println!("{}: command not found", cmd),
                    }
                }
            }
        }
        true
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}

