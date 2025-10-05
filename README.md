# <p align="center">🚀 Shelly</p>

<p align="center">
    <em>A standalone, POSIX-compliant shell written in Rust.</em>
</p>

<p align="center">
    <img src="https://img.shields.io/github/license/botirk38/shelly?style=default&logo=opensourceinitiative&logoColor=white&color=0080ff" alt="license">
    <img src="https://img.shields.io/github/last-commit/botirk38/shelly?style=default&logo=git&logoColor=white&color=0080ff" alt="last-commit">
    <img src="https://img.shields.io/github/languages/top/botirk38/shelly?style=default&color=0080ff" alt="repo-top-language">
    <img src="https://img.shields.io/github/languages/count/botirk38/shelly?style=default&color=0080ff" alt="repo-language-count">
</p>

## Project Overview
### Description
Shelly is a fully standalone, POSIX-compliant shell written in Rust. It is designed to be a lightweight yet powerful alternative to traditional shells, providing fast execution and robust scripting capabilities. Unlike many shell implementations, Shelly does not rely on external dependencies for core functionality, ensuring high performance and portability.

### Key Features
* **Standalone Implementation**: Does not rely on existing shell interpreters or external runtime dependencies.
* **POSIX Compliance**: Adheres to the POSIX shell standard, ensuring compatibility with shell scripts and common UNIX utilities.
* **Rust-Based**: Built entirely in Rust for safety, speed, and reliability.
* **Extensible and Modular**: Designed for easy integration with other Rust projects and custom extensions.
* **Efficient Execution**: Optimized for minimal overhead and fast command execution.

## Quick Start
To get started with Shelly, follow these steps:
1. Clone the repository: `git clone https://github.com/botirk38/shelly.git`
2. Change into the project directory: `cd shelly`
3. Build the project: `cargo build`
4. Run the shell: `cargo run`

## Installation
### Dependencies
* **Rust**: Install Rust (version 1.64 or later) on your system.
* **Cargo**: Ensure Cargo (version 1.64 or later) is available as the package manager.

### System Requirements
* Compatible operating systems: Linux, macOS, Windows (with a UNIX-like environment)
* Recommended: A terminal emulator that supports ANSI escape sequences

### Setup Steps
1. Clone the repository: `git clone https://github.com/botirk38/shelly.git`
2. Change into the project directory: `cd shelly`
3. Build the project: `cargo build`
4. Install the shell globally (optional): `cargo install --path .`

## Usage Examples
### Basic Commands
* Run a simple command: `echo "Hello, World!"`
* List directory contents: `ls -l`
* Change directory: `cd /path/to/directory`
* Execute a script: `./script.sh`

### Advanced Usage
* Use Shelly as the default shell: `chsh -s $(which shelly)`
* Create shell scripts with POSIX syntax and execute them using Shelly.
* Utilize built-in shell functions and scripting features for automation.

## API Docs
Currently, there is no structured API reference available. However, detailed documentation on built-in commands and shell behavior will be provided in future updates.

## Build & Deployment
### Local Development
To build and run Shelly locally:
1. Clone the repository: `git clone https://github.com/botirk38/shelly.git`
2. Change into the project directory: `cd shelly`
3. Build the project: `cargo build`
4. Run the shell: `cargo run`

### Testing
* Run all tests: `cargo test`
* Run specific test: `cargo test <test_name>`

### Deployment
To deploy Shelly, you can:
* Build a release binary: `cargo build --release` and distribute it as a standalone executable.
* Use Docker: Build a Docker image and distribute it in containerized environments.

## Contribution Guide
### Contributing
Contributions are welcome! Follow these steps:
1. Fork the repository
2. Create a new branch (`git checkout -b feature-branch`)
3. Commit your changes (`git commit -m "Add feature"`)
4. Push to the branch (`git push origin feature-branch`)
5. Open a pull request

### Code of Conduct
* Be respectful and considerate of others.
* Follow best practices for Rust development and POSIX compliance.
* Ensure your contributions are well-documented and tested.

### License
This project is licensed under the [MIT License](https://github.com/botirk38/shelly/blob/main/LICENSE).

