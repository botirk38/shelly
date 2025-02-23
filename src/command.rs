use std::path::PathBuf;

#[derive(Debug, PartialEq)]
enum Token {
    Word(String),
    OutputRedirect(bool), // bool indicates append mode
    ErrorRedirect(bool),  // bool indicates append mode
    Pipe,
    Background,
}

#[derive(Debug)]
pub struct CommandParts {
    pub command: String,
    pub args: Vec<String>,
    pub output_redirect: Option<(PathBuf, bool)>,
    pub error_redirect: Option<(PathBuf, bool)>,
}

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

    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.position < self.chars.len() {
            let ch = self.chars[self.position];
            self.position += 1;
            Some(ch)
        } else {
            None
        }
    }

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

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' => {
                    self.advance();
                }

                '>' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::OutputRedirect(true)); // append mode
                    } else {
                        tokens.push(Token::OutputRedirect(false)); // overwrite mode
                    }
                }

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
                        tokens.push(Token::Word("1".to_string()));
                    }
                }

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
                        tokens.push(Token::Word("2".to_string()));
                    }
                }

                '|' => {
                    self.advance();
                    tokens.push(Token::Pipe);
                }
                '&' => {
                    self.advance();
                    tokens.push(Token::Background);
                }
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

pub struct CommandParser;

impl CommandParser {
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

        while let Some(token) = tokens_iter.next() {
            match token {
                Token::Word(word) => {
                    if command_parts.command.is_empty() {
                        command_parts.command = word;
                    } else {
                        command_parts.args.push(word);
                    }
                }
                Token::OutputRedirect(append) => {
                    if let Some(Token::Word(path)) = tokens_iter.next() {
                        command_parts.output_redirect = Some((PathBuf::from(path), append));
                    }
                }
                Token::ErrorRedirect(append) => {
                    if let Some(Token::Word(path)) = tokens_iter.next() {
                        command_parts.error_redirect = Some((PathBuf::from(path), append));
                    }
                }
                _ => {}
            }
        }

        command_parts
    }
}
