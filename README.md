# <p align="center">🚀 Shelly</p>

<p align="center">
    <em>A basic interactive shell written in Rust</em>
</p>

<p align="center">
    <img src="https://img.shields.io/github/license/botirk38/shelly?style=default&logo=opensourceinitiative&logoColor=white&color=0080ff" alt="license">
    <img src="https://img.shields.io/github/last-commit/botirk38/shelly?style=default&logo=git&logoColor=white&color=0080ff" alt="last-commit">
    <img src="https://img.shields.io/github/languages/top/botirk38/shelly?style=default&color=0080ff" alt="repo-top-language">
    <img src="https://img.shields.io/github/languages/count/botirk38/shelly?style=default&color=0080ff" alt="repo-language-count">
</p>

## Project Overview

### Description
Shelly is a basic interactive shell written in Rust, designed as a learning project to explore shell implementation concepts. It provides fundamental command execution capabilities with a focus on code quality and Rust best practices.

### Key Features
* **Basic Command Execution**: Run external commands and a small set of built-in commands
* **I/O Redirection**: Support for output redirection (`>`, `>>`) and error redirection (`2>`, `2>>`)
* **Tab Completion**: Intelligent command completion using a Trie-based algorithm
* **Command History**: Persistent command history across sessions
* **Rust-Based**: Built entirely in Rust for safety, speed, and reliability
* **Quote Handling**: Support for single and double quotes with escape sequences

### Supported Built-in Commands
* `cd` - Change directory (with `~` expansion)
* `echo` - Print arguments to stdout
* `pwd` - Print working directory
* `exit` - Exit the shell with optional status code
* `type` - Determine if a command is a builtin or show its path
* `history` - Command history (managed by rustyline)

### Known Limitations
This is a basic shell implementation and does **not** support:
* Pipes (`|`)
* Background jobs (`&`)
* Shell variables and environment variable expansion (`$VAR`)
* Command substitution (`$(...)` or backticks)
* Conditional execution (`&&`, `||`, `;`)
* Globbing (`*`, `?`, `[...]`)
* Control flow (`if`, `while`, `for`, `case`)
* Shell functions
* Input redirection (`<`, `<<`)
* Script file execution
* Most POSIX shell features

**This shell is not suitable for use as a default shell or for running shell scripts.**

## Quick Start

### Prerequisites
* Rust 1.80.0 or later
* Cargo (comes with Rust)

### Build and Run
```bash
# Clone the repository
git clone https://github.com/botirk38/shelly.git
cd shelly

# Build the project
cargo build

# Run the shell
cargo run
```

### Install Locally
```bash
# Install to ~/.cargo/bin
cargo install --path .

# Run the installed binary
shelly
```

## Development

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Check formatting without modifying files
cargo fmt -- --check
```

## Usage Examples

### Interactive Mode
```bash
$ echo "Hello, World!"
Hello, World!

$ cd /tmp
$ pwd
/tmp

$ ls -la
# Lists files (runs external ls command)

$ echo "test" > output.txt
# Redirects output to file

$ type echo
echo is a shell builtin

$ type ls
ls is /usr/bin/ls

$ exit
```

### Supported Redirection
```bash
# Output redirection (overwrite)
echo "text" > file.txt

# Output redirection (append)
echo "more text" >> file.txt

# Error redirection
command_that_fails 2> errors.txt

# Error redirection (append)
command_that_fails 2>> errors.txt
```

## Architecture

See [CLAUDE.md](CLAUDE.md) for detailed architecture documentation.

### Project Structure
```
src/
├── main.rs         # Entry point
├── lib.rs          # Library exports
├── shell.rs        # Main shell REPL and command execution
├── command.rs      # Lexer and parser for command parsing
├── builtin.rs      # Built-in command implementations
├── completion.rs   # Tab completion using Trie data structure
└── error.rs        # Error types
```

## Contributing

Contributions are welcome! This is a learning project, so feel free to:
* Add new built-in commands
* Implement additional shell features
* Improve error handling
* Add more tests
* Fix bugs

### Contribution Steps
1. Fork the repository
2. Create a feature branch (`git checkout -b feature-name`)
3. Make your changes
4. Run tests and linting (`cargo test && cargo clippy`)
5. Format your code (`cargo fmt`)
6. Commit your changes (`git commit -m "Description"`)
7. Push to your fork (`git push origin feature-name`)
8. Open a pull request

### Code Standards
* Follow Rust best practices and idioms
* Write tests for new functionality
* Document public APIs with doc comments
* Ensure code passes `cargo clippy` and `cargo fmt`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built as part of the [CodeCrafters](https://codecrafters.io) Shell challenge.

## Support

For issues or questions:
* Open an issue on GitHub
* Check existing issues for solutions
