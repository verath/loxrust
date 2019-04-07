use std::str;

use super::token::{Literal, Token, TokenType};
use super::ErrorCallback;

pub struct Scanner<'a> {
    source: &'a [u8],

    tokens: Vec<Token>,

    // had_error is set to true if any error is encountered while scanning.
    had_error: bool,

    // error_cb is an optional ErrorCallback that will be notified for each
    // (if any) errors encountered while scanning.
    error_cb: Option<&'a ErrorCallback>,

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

    // report_error reports an error on the current line with the provided
    // msg to the registered error_cb. report_error also sets the had_error
    // flag.
    fn report_error(&mut self, msg: &str) {
        self.had_error = true;
        if let Some(f) = self.error_cb {
            f(self.line, msg)
        }
    }

    // scan_tokens scans the source for tokens returning a tuple (had_error, tokens)
    // where had_error is false only if all characters in source were successfully
    // consumed, and tokens is the successfully scanned tokens.
    pub fn scan_tokens(&mut self) -> (bool, impl IntoIterator<Item = &Token> + '_) {
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
        (self.had_error, &self.tokens)
    }

    pub fn new<'a, 'e: 'a>(source: &'a str, error_cb: Option<&'e ErrorCallback>) -> Scanner<'a> {
        Scanner {
            source: source.as_bytes(),
            tokens: Vec::new(),
            had_error: false,
            error_cb,
            start: 0,
            current: 0,
            line: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panic_on_error(line: u64, msg: &str) {
        panic!("error: '{line}:{msg}'", line = line, msg = msg);
    }

    #[test]
    fn test_scan_tokens_appends_eof() {
        let source = "";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (_, tokens) = scanner.scan_tokens();
        let mut token_types = tokens.into_iter().map(|t| t.token_type);
        assert_eq!(token_types.next(), Some(TokenType::Eof));
        assert_eq!(token_types.next(), None);
    }

    #[test]
    fn test_scan_simple_tokens() {
        let source = "( ) { } , . - + ; / * ! != = == > >= < <=";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (_, tokens) = scanner.scan_tokens();
        let mut tokens = tokens.into_iter();

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
        // One char tokens.
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
        // One or two char tokens.
        assert_eq!(tokens.next(), Some(&make_token(Bang, "!")));
        assert_eq!(tokens.next(), Some(&make_token(BangEqual, "!=")));
        assert_eq!(tokens.next(), Some(&make_token(Equal, "=")));
        assert_eq!(tokens.next(), Some(&make_token(EqualEqual, "==")));
        assert_eq!(tokens.next(), Some(&make_token(Greater, ">")));
        assert_eq!(tokens.next(), Some(&make_token(GreaterEqual, ">=")));
        assert_eq!(tokens.next(), Some(&make_token(Less, "<")));
        assert_eq!(tokens.next(), Some(&make_token(LessEqual, "<=")));

        assert_eq!(tokens.next(), Some(&make_token(Eof, "")));
    }

    #[test]
    fn test_scan_identifer() {
        let source = " abc _def gHiJ kl_mn a1 0a ";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (_, tokens) = scanner.scan_tokens();
        let mut tokens = tokens.into_iter();

        fn make_identifer_token(identifier: &str) -> Token {
            let lexeme = identifier.to_owned();
            Token {
                token_type: TokenType::Identifier,
                lexeme,
                line: 1,
                literal: None,
            }
        }

        assert_eq!(tokens.next(), Some(&make_identifer_token("abc")));
        assert_eq!(tokens.next(), Some(&make_identifer_token("_def")));
        assert_eq!(tokens.next(), Some(&make_identifer_token("gHiJ")));
        assert_eq!(tokens.next(), Some(&make_identifer_token("kl_mn")));
        assert_eq!(tokens.next(), Some(&make_identifer_token("a1")));
        assert_eq!(tokens.next().map(|t| t.token_type), Some(TokenType::Number));
        assert_eq!(tokens.next(), Some(&make_identifer_token("a")));
        assert_eq!(tokens.next().map(|t| t.token_type), Some(TokenType::Eof));
    }

    #[test]
    fn test_scan_keyword() {
        let source = " for IF force ";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (_, tokens) = scanner.scan_tokens();
        let mut token_types = tokens.into_iter().map(|t| t.token_type);

        assert_eq!(token_types.next(), Some(TokenType::For));
        assert_eq!(token_types.next(), Some(TokenType::Identifier));
        assert_eq!(token_types.next(), Some(TokenType::Identifier));
        assert_eq!(token_types.next(), Some(TokenType::Eof));
    }

    #[test]
    fn test_scan_string() {
        let source = " \"ab\" \"c\nd\" \"ef\" ";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (_, tokens) = scanner.scan_tokens();
        let mut tokens = tokens.into_iter();

        fn make_string_token(s: &str, line: u64) -> Token {
            let lexeme = format!(r#""{}""#, s); // Add quotes.
            let literal = Some(Literal::String(s.to_owned()));
            Token {
                token_type: TokenType::String,
                lexeme,
                line,
                literal,
            }
        }

        assert_eq!(tokens.next(), Some(&make_string_token("ab", 1)));
        assert_eq!(tokens.next(), Some(&make_string_token("c\nd", 2)));
        assert_eq!(tokens.next(), Some(&make_string_token("ef", 2)));
        assert_eq!(tokens.next().map(|t| t.token_type), Some(TokenType::Eof));
    }

    #[test]
    fn test_scan_number() {
        let source = " 111 111.222 -333 444. ";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (_, tokens) = scanner.scan_tokens();
        let mut tokens = tokens.into_iter();

        fn make_number_token(n: f64) -> Token {
            let lexeme = format!("{}", n);
            let literal = Some(Literal::Number(n));
            Token {
                token_type: TokenType::Number,
                lexeme,
                line: 1,
                literal,
            }
        }

        assert_eq!(tokens.next(), Some(&make_number_token(111.0)));
        assert_eq!(tokens.next(), Some(&make_number_token(111.222)));
        assert_eq!(tokens.next().map(|t| t.token_type), Some(TokenType::Minus));
        assert_eq!(tokens.next(), Some(&make_number_token(333.0)));
        assert_eq!(tokens.next(), Some(&make_number_token(444.0)));
        assert_eq!(tokens.next().map(|t| t.token_type), Some(TokenType::Dot));
        assert_eq!(tokens.next().map(|t| t.token_type), Some(TokenType::Eof));
    }

    #[test]
    #[should_panic(expected = "2:Unexpected character '~'.")]
    fn test_scan_tokens_unexpected_token() {
        let source = "\n~";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        scanner.scan_tokens();
    }

    #[test]
    #[should_panic(expected = "3:Unterminated string.")]
    fn test_scan_tokens_unterminated_string() {
        let source = "\n\"\n";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        scanner.scan_tokens();
    }

    #[test]
    fn test_had_error_ok_scan() {
        let source = "";
        let mut scanner = Scanner::new(source, Some(&panic_on_error));
        let (had_error, _) = scanner.scan_tokens();
        assert_eq!(had_error, false);
    }

    #[test]
    fn test_had_error_failed_scan() {
        let source = "~"; // Unexpected token '~'.
        let mut scanner = Scanner::new(source, None);
        let (had_error, _) = scanner.scan_tokens();
        assert_eq!(had_error, true);
    }

}
