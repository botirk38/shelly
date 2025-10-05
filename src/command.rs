use std::path::PathBuf;

/// Tokens produced by the lexer during command parsing
#[derive(Debug, PartialEq)]
enum Token {
    /// A word or argument (handles quoted strings and escape sequences)
    Word(String),
    /// Output redirection (>, >> or 1>, 1>>). Bool indicates append mode
    OutputRedirect(bool),
    /// Error redirection (2>, 2>>). Bool indicates append mode
    ErrorRedirect(bool),
    /// Pipe operator (|)
    Pipe,
    /// Background operator (&)
    Background,
}

/// Parsed command with its arguments and redirections
#[derive(Debug)]
pub struct CommandParts {
    /// The command name
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Output redirection (file path, append mode)
    pub output_redirect: Option<(PathBuf, bool)>,
    /// Error redirection (file path, append mode)
    pub error_redirect: Option<(PathBuf, bool)>,
}

/// Lexer that tokenizes shell command input
struct Lexer {
    position: usize,
    chars: Vec<char>,
}

impl Lexer {
    fn new(input: String) -> Self {
        Self {
            position: 0,
            chars: input.chars().collect(),
        }
    }

    /// Peek at the current character without consuming it
    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    /// Advance to the next character and return the current one
    fn advance(&mut self) -> Option<char> {
        if self.position < self.chars.len() {
            let ch = self.chars[self.position];
            self.position += 1;
            Some(ch)
        } else {
            None
        }
    }

    /// Read a word, handling quotes and escape sequences
    /// Supports single quotes (literal), double quotes (with escapes), and backslash escaping
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        let mut in_quotes = None;

        while let Some(ch) = self.peek() {
            match ch {
                '"' | '\'' => {
                    self.advance();
                    match in_quotes {
                        None => in_quotes = Some(ch),
                        Some(quote) if quote == ch => in_quotes = None,
                        Some(_) => word.push(ch),
                    }
                }
                '\\' => {
                    self.advance();
                    match in_quotes {
                        None => {
                            if let Some(next) = self.advance() {
                                match next {
                                    'n' => word.push('n'),
                                    _ => word.push(next),
                                }
                            }
                        }
                        Some(quote_char) => {
                            word.push('\\');
                            if let Some(next) = self.advance() {
                                if quote_char == '"' && (next == '"' || next == '\\') {
                                    word.pop();
                                }
                                word.push(next);
                            }
                        }
                    }
                }
                ' ' | '\t' if in_quotes.is_none() => break,
                _ => {
                    word.push(ch);
                    self.advance();
                }
            }
        }
        word
    }

    /// Tokenize the input string into a sequence of tokens
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                // Skip whitespace
                ' ' | '\t' => {
                    self.advance();
                }

                // Handle output redirection: > or >>
                '>' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::OutputRedirect(true)); // append mode
                    } else {
                        tokens.push(Token::OutputRedirect(false)); // overwrite mode
                    }
                }

                // Handle explicit stdout redirection: 1> or 1>>
                '1' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        if self.peek() == Some('>') {
                            self.advance();
                            tokens.push(Token::OutputRedirect(true)); // append mode
                        } else {
                            tokens.push(Token::OutputRedirect(false)); // overwrite mode
                        }
                    } else {
                        // Just the number "1", not a redirect
                        tokens.push(Token::Word("1".to_string()));
                    }
                }

                // Handle stderr redirection: 2> or 2>>
                '2' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        if self.peek() == Some('>') {
                            self.advance();
                            tokens.push(Token::ErrorRedirect(true));
                        } else {
                            tokens.push(Token::ErrorRedirect(false));
                        }
                    } else {
                        // Just the number "2", not a redirect
                        tokens.push(Token::Word("2".to_string()));
                    }
                }

                // Pipe operator
                '|' => {
                    self.advance();
                    tokens.push(Token::Pipe);
                }
                // Background operator
                '&' => {
                    self.advance();
                    tokens.push(Token::Background);
                }
                // Regular word or argument
                _ => {
                    let word = self.read_word();
                    if !word.is_empty() {
                        tokens.push(Token::Word(word));
                    }
                }
            }
        }
        tokens
    }
}

/// Parser that converts tokens into a structured command representation
pub struct CommandParser;

impl CommandParser {
    /// Parse a command line string into CommandParts
    ///
    /// # Examples
    /// ```
    /// use codecrafters_shell::command::CommandParser;
    ///
    /// let cmd = CommandParser::parse("echo hello > output.txt");
    /// assert_eq!(cmd.command, "echo");
    /// assert_eq!(cmd.args, vec!["hello"]);
    /// assert!(cmd.output_redirect.is_some());
    /// ```
    pub fn parse(input: &str) -> CommandParts {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize();

        let mut command_parts = CommandParts {
            command: String::new(),
            args: Vec::new(),
            output_redirect: None,
            error_redirect: None,
        };

        let mut tokens_iter = tokens.into_iter().peekable();

        // Process tokens to build command structure
        while let Some(token) = tokens_iter.next() {
            match token {
                Token::Word(word) => {
                    // First word is the command, rest are arguments
                    if command_parts.command.is_empty() {
                        command_parts.command = word;
                    } else {
                        command_parts.args.push(word);
                    }
                }
                Token::OutputRedirect(append) => {
                    // Next token should be the file path
                    if let Some(Token::Word(path)) = tokens_iter.next() {
                        command_parts.output_redirect = Some((PathBuf::from(path), append));
                    }
                }
                Token::ErrorRedirect(append) => {
                    // Next token should be the file path
                    if let Some(Token::Word(path)) = tokens_iter.next() {
                        command_parts.error_redirect = Some((PathBuf::from(path), append));
                    }
                }
                // Pipe and Background tokens are recognized but not yet handled
                _ => {}
            }
        }

        command_parts
    }
}
