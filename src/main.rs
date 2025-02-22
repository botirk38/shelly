use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static TAB_PRESSED: AtomicBool = AtomicBool::new(false);
static LAST_TAB_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Debug)]
struct Shell {
    current_dir: PathBuf,
    builtin_commands: HashSet<String>,
    editor: Editor<RustylineHelper, FileHistory>,
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

        let helper = RustylineHelper {
            completer: RustylineCompleter,
        };
        let mut editor = Editor::new().unwrap();
        editor.set_helper(Some(helper));

        Shell {
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            builtin_commands,
            editor,
        }
    }

    fn run(&mut self) {
        loop {
            let prompt = "$ ";

            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.trim();

                    if !line.is_empty() {
                        let _ = self.editor.add_history_entry(line);

                        if !self.execute_command(line) {
                            break;
                        }
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
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
            "history" => {}
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

struct RustylineCompleter;
#[derive(Helper, Completer, Hinter, Highlighter, Validator)]
struct RustylineHelper {
    #[rustyline(Completer)]
    completer: RustylineCompleter,
}
impl rustyline::completion::Completer for RustylineCompleter {
    type Candidate = String;
    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut matches = Vec::new();

        let builtins = ["echo", "type", "exit", "pwd", "cd", "history"];
        matches.extend(
            builtins
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|s| s.to_string()),
        );

        // Find executables from PATH that match the prefix
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.filter_map(Result::ok) {
                        if let Ok(name) = entry.file_name().into_string() {
                            if name.starts_with(line) {
                                matches.push(name);
                            }
                        }
                    }
                }
            }
        }

        matches.sort();
        matches.dedup();

        if matches.is_empty() {
            return Ok((0, vec![]));
        }

        if matches.len() == 1 {
            return Ok((0, vec![matches[0].clone() + " "]));
        }

        // Find longest common prefix
        let mut common_prefix = matches[0].clone();
        for name in &matches[1..] {
            while !name.starts_with(&common_prefix) {
                common_prefix.pop();
            }
        }

        // Always return the common prefix on first tab if it's longer than current input
        if common_prefix.len() > line.len() {
            return Ok((0, vec![common_prefix]));
        }

        // Show all matches only on double-tab
        let now = Instant::now().elapsed().as_millis() as u64;
        let last_tab = LAST_TAB_TIME.load(Ordering::Relaxed);

        if now - last_tab < 500 {
            println!("\n{}", matches.join("  "));
            print!("$ {}", line);
            std::io::stdout().flush().unwrap();
            TAB_PRESSED.store(false, Ordering::Relaxed);
        } else {
            TAB_PRESSED.store(true, Ordering::Relaxed);
        }

        LAST_TAB_TIME.store(now, Ordering::Relaxed);
        Ok((0, vec![]))
    }
}
