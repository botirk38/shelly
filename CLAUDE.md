# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Shelly is a POSIX-compliant shell written in Rust. It's a standalone implementation that does not rely on external shell interpreters or runtime dependencies.

## Development Commands

### Build and Run
- **Build the project**: `cargo build`
- **Run the shell**: `cargo run`
- **Build release binary**: `cargo build --release`

### Testing
- **Run all tests**: `cargo test`
- **Run specific test**: `cargo test <test_name>`
- **Run tests without executing**: `cargo test --no-run`

## Architecture

### Core Components

**Shell Execution Flow** (`shell.rs`):
- `Shell` struct is the main entry point, initialized via `Shell::new()`
- Uses `rustyline` for interactive input with `Editor` and custom `RustylineHelper`
- Maintains `current_dir` state and `BuiltinRegistry` for built-in commands
- Command execution flow: `run()` → `execute_command()` → `execute_builtin()` or `execute_external()`
- History is persisted to `history.txt` file

**Command Parsing** (`command.rs`):
- Two-stage parsing: `Lexer` tokenizes input, then `CommandParser` builds `CommandParts`
- `Lexer` handles: quotes (single/double), escape sequences, redirects (`>`, `>>`, `2>`, `2>>`), pipes, and background operators
- `CommandParts` captures: command name, arguments, output redirect, and error redirect
- Quote handling: double quotes allow escape sequences, single quotes are literal

**Built-in Commands** (`builtin.rs`):
- Uses trait-based plugin architecture via `BuiltinCommand` trait
- Commands registered in `BuiltinRegistry` (HashMap-based lookup)
- Current built-ins: `cd`, `echo`, `pwd`, `exit`, `type`, `history`
- To add new built-in: implement `BuiltinCommand` trait and register in `BuiltinRegistry::new()`
- `TypeCommand` checks both built-ins and PATH executables

**Tab Completion** (`completion.rs`):
- Trie-based completion engine for performance with large PATH
- Completes both built-in commands and executables in PATH
- Double-tab within 500ms shows all matches, otherwise completes common prefix
- `CompletionEngine` caches all available commands in a Trie structure
- Integrated with `rustyline` via `RustylineHelper` which implements `Completer` trait

**Error Handling** (`error.rs`):
- Centralized error types in `ShellError` enum
- Implements `std::error::Error` and `Display` traits
- Auto-conversion from `io::Error` via `From` trait

### Module Structure
- `main.rs`: Entry point, initializes Shell
- `lib.rs`: Public module declarations
- All modules are re-exported through lib.rs for use as a library

## Key Implementation Details

### Command Execution
Built-in commands are checked first via `BuiltinRegistry::is_builtin()`. If not built-in, spawns external process via `std::process::Command`. The shell tracks working directory state separately from external commands.

### I/O Redirection
Both built-in and external commands support:
- Output redirect: `>` (overwrite), `>>` (append), `1>`, `1>>`
- Error redirect: `2>`, `2>>`
Redirects are parsed into `CommandParts` and handled during execution.

### State Management
The `Shell` maintains:
- `current_dir`: working directory (updated after successful `cd`)
- `builtin_registry`: command lookup registry
- `editor`: rustyline editor with history and completion

## Dependencies
- `rustyline`: Interactive line editing and history
- `anyhow`/`thiserror`: Error handling
- `bytes`: Buffer management
- `log`/`env_logger`: Logging infrastructure
