use std::str;

use super::token::{Literal, Token, TokenType};
use super::ErrorCallback;

pub struct Scanner<'a> {
    source: &'a [u8],

    tokens: Vec<Token>,

    error: bool,

    error_cb: &'a ErrorCallback,

    // start is the offset in source of the first character of the
    // lexeme we are currently considering.
    start: usize,
    // current is the offset in source of the current character of the
    // lexeme we are currently considering.
    current: usize,

    // line is the line number of the current lexeme.
    line: u64,
}

impl Scanner<'_> {
    fn is_digit(ch: char) -> bool {
        ch.is_digit(10)
    }

    fn is_alpha(ch: char) -> bool {
        (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_'
    }

    fn is_alpha_numeric(ch: char) -> bool {
        Scanner::is_digit(ch) || Scanner::is_alpha(ch)
    }

    fn is_whitespace(ch: char) -> bool {
        ch.is_ascii_whitespace()
    }

    // keyword returns the TokenType for the keyword_str, or None
    // if no keyword matched the str.
    fn keyword(keyword_str: &str) -> Option<TokenType> {
        match keyword_str {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "fun" => Some(TokenType::Fun),
            "for" => Some(TokenType::For),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    // add_token creates a token from the current lexeme.
    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = &self.source[self.start..self.current];
        let lexeme = str::from_utf8(lexeme).unwrap().to_owned();
        let line = self.line;
        self.tokens.push(Token {
            token_type,
            lexeme,
            line,
            literal,
        });
    }

    // peek_next returns the character following the next character in the source
    // without consuming it.
    fn peek_next(&self) -> char {
        if (self.current + 1) >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1] as char
        }
    }

    // peek returns the next character in the source without consuming it.
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current] as char
        }
    }

    // advance_if consumes the next character in the source if the character
    // matches expected.
    // Note this method is called "match" in lox/Scanner.java, but match is
    // reserved in rust.
    fn advance_if(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        let ch = self.source[self.current] as char;
        if ch == expected {
            self.current += 1;
            true
        } else {
            false
        }
    }

    // advance consumes the next character in the source and returns it.
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1] as char
    }

    // consume_line consumes characters until it encounters a newline
    // character ('\n') or end of source.
    fn consume_line(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    // string consumes a string, producing a String token.
    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // Unterminated string.
        if self.is_at_end() {
            self.report_error("Unterminated string.");
            return;
        }

        // Closing '"'.
        self.advance();

        // Trim surrounding quotes.
        let value = &self.source[(self.start + 1)..(self.current - 1)];
        // Convert to owned String
        let value = str::from_utf8(value).unwrap();
        let value = Literal::String(value.to_owned());
        self.add_token(TokenType::String, Some(value));
    }

    // number consumes a number, producing a Number token.
    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        // Possibly a decimal number.
        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            // Consume the '.'
            self.advance();
            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = &self.source[(self.start)..(self.current)];
        let value = str::from_utf8(value).unwrap();
        let value: f64 = value.parse().unwrap();
        let value = Literal::Number(value);
        self.add_token(TokenType::Number, Some(value));
    }

    // identifier consumes an identifier, producing an Identifier token.
    // If the identifier matches a reserved keyword a token for that
    // matched keyword is produced instead.
    fn identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        // Test for reserved keyword.
        let text = &self.source[(self.start)..(self.current)];
        let text = str::from_utf8(text).unwrap();
        let token_type = Scanner::keyword(text).unwrap_or(TokenType::Identifier);
        self.add_token(token_type, None);
    }

    // scan_token scans a single token.
    fn scan_token(&mut self) {
        let ch = self.advance();
        let tok_type: Option<TokenType> = match ch {
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            ',' => Some(TokenType::Comma),
            '.' => Some(TokenType::Dot),
            '-' => Some(TokenType::Minus),
            '+' => Some(TokenType::Plus),
            ';' => Some(TokenType::Semicolon),
            '*' => Some(TokenType::Star),
            '!' => {
                if self.advance_if('=') {
                    Some(TokenType::BangEqual)
                } else {
                    Some(TokenType::Bang)
                }
            }
            '=' => {
                if self.advance_if('=') {
                    Some(TokenType::EqualEqual)
                } else {
                    Some(TokenType::Equal)
                }
            }
            '<' => {
                if self.advance_if('=') {
                    Some(TokenType::LessEqual)
                } else {
                    Some(TokenType::Less)
                }
            }
            '>' => {
                if self.advance_if('=') {
                    Some(TokenType::GreaterEqual)
                } else {
                    Some(TokenType::Greater)
                }
            }
            '/' => {
                if self.advance_if('/') {
                    // Comments continue until end of line.
                    self.consume_line();
                    None
                } else {
                    Some(TokenType::Slash)
                }
            }
            '\n' => {
                self.line += 1;
                None
            }
            '"' => {
                self.string();
                None // self.string handles adding token.
            }
            _ if Scanner::is_digit(ch) => {
                self.number();
                None // self.number handles adding token.
            }
            _ if Scanner::is_alpha(ch) => {
                self.identifier();
                None // self.identifier handles adding token.
            }
            _ if Scanner::is_whitespace(ch) => None, // Ignore whitespace.
            _ => {
                self.report_error(&format!("Unexpected character '{}'.", ch));
                None
            }
        };
        if let Some(tok_type) = tok_type {
            self.add_token(tok_type, None)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn report_error(&mut self, msg: &str) {
        self.error = true;
        (self.error_cb)(self.line, msg)
    }

    pub fn had_error(&self) -> bool {
        self.error
    }

    pub fn scan_tokens(&mut self) -> impl IntoIterator<Item = &Token> + '_ {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token()
        }
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::from(""),
            line: self.line,
            literal: None,
        });
        &self.tokens
    }

    pub fn new<'a, 'e: 'a>(source: &'a str, error_cb: &'e ErrorCallback) -> Scanner<'a> {
        Scanner {
            source: source.as_bytes(),
            tokens: Vec::new(),
            error: false,
            error_cb,
            start: 0,
            current: 0,
            line: 1,
        }
    }
}

mod tests {
    use super::*;

    #[allow(dead_code)]
    fn dummy_handle_err(_line: u64, _msg: &str) {}

    #[test]
    fn test_scan_tokens_appends_eof() {
        let source = "";
        let mut scanner = Scanner::new(source, &dummy_handle_err);
        let mut token_types: Vec<TokenType> = scanner
            .scan_tokens()
            .into_iter()
            .map(|t| t.token_type)
            .collect();
        assert_eq!(token_types.pop(), Some(TokenType::Eof));
        assert_eq!(token_types.pop(), None);
    }

    #[test]
    fn test_scan_tokens_single_char_tokens() {
        let source = "(){},.-+;/*";
        let mut scanner = Scanner::new(source, &dummy_handle_err);
        let mut tokens = scanner.scan_tokens().into_iter();

        fn make_token(token_type: TokenType, lexeme: &str) -> Token {
            let lexeme = lexeme.to_owned();
            Token {
                token_type,
                lexeme,
                line: 1,
                literal: None,
            }
        }

        use TokenType::*;
        assert_eq!(tokens.next(), Some(&make_token(LeftParen, "(")));
        assert_eq!(tokens.next(), Some(&make_token(RightParen, ")")));
        assert_eq!(tokens.next(), Some(&make_token(LeftBrace, "{")));
        assert_eq!(tokens.next(), Some(&make_token(RightBrace, "}")));
        assert_eq!(tokens.next(), Some(&make_token(Comma, ",")));
        assert_eq!(tokens.next(), Some(&make_token(Dot, ".")));
        assert_eq!(tokens.next(), Some(&make_token(Minus, "-")));
        assert_eq!(tokens.next(), Some(&make_token(Plus, "+")));
        assert_eq!(tokens.next(), Some(&make_token(Semicolon, ";")));
        assert_eq!(tokens.next(), Some(&make_token(Slash, "/")));
        assert_eq!(tokens.next(), Some(&make_token(Star, "*")));
        assert_eq!(tokens.next(), Some(&make_token(Eof, "")));
    }
}
