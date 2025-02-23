use codecrafters_shell::shell::Shell;

fn main() {
    match Shell::new() {
        Ok(mut shell) => {
            if let Err(e) = shell.run() {
                eprintln!("Shell error: {:?}", e);
            }
        }
        Err(e) => eprintln!("Failed to initialize shell: {:?}", e),
    }
}

