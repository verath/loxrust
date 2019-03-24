#[derive(Debug)]
pub enum TokenType {
    EOF,
}

#[derive(Debug)]
pub struct Token {
    tok_type: TokenType,
}

pub struct Scanner<'a> {
    source: &'a str,
}

impl Scanner<'_> {
    pub fn new(source: &str) -> Scanner {
        Scanner { source }
    }

    pub fn scan_tokens(&mut self) -> impl IntoIterator<Item = Token> + '_ {
        let chars = self.source.chars();
        chars.map(|_c| Token {
            tok_type: TokenType::EOF,
        })
    }
}
