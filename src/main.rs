use std::env::args;
use std::error::Error;
use std::fs::read;

use lexer::Lexer;

fn main() -> Result<(), Box<dyn Error>> {
    let contents;
    if let Some(file) = args().nth(1) {
        contents = read(file)?;
    } else {
        panic!("No files given");
    }
    let mut lexer = Lexer::new(&contents);
    lexer.lex().unwrap();
    let tokens = lexer.get_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}

pub mod lexer {
    use std::borrow::Cow;

    pub struct Lexer<'a> {
        input: &'a [u8],
        tokens: Vec<Token<'a>>,
    }

    impl<'a> Lexer<'a> {
        pub fn new(input: &'a [u8]) -> Self {
            Self {
                input,
                tokens: vec![],
            }
        }

        pub fn get_tokens(self) -> Vec<Token<'a>> {
            self.tokens
        }

        pub fn lex(&mut self) -> Result<(), LexError> {
            while let Some(&ch) = self.input.first() {
                match ch {
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                        let token = self.lex_word()?;
                        self.tokens.push(token);
                    }
                    b'0'..=b'9' | b'-' => {
                        let token = self.lex_number()?;
                        self.tokens.push(token);
                    }
                    b'"' => {
                        let token = self.lex_string()?;
                        self.tokens.push(token);
                    }
                    c if c.is_ascii_whitespace() => {
                        self.input = &self.input[1..];
                    }
                    // symbols and stuff
                    _ => panic!(),
                }
            }
            Ok(())
        }

        fn lex_word(&mut self) -> Result<Token<'a>, LexError> {
            let idx = self
                .input
                .iter()
                .enumerate()
                .find_map(|(idx, &ch)| {
                    if !matches!(ch, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9') {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .unwrap_or(self.input.len());
            let word = std::str::from_utf8(&self.input[..idx]).unwrap();
            self.input = &self.input[idx..];
            let token = match word {
                "import" => Token::Keyword(Keyword::Import),
                "struct" => Token::Keyword(Keyword::Struct),
                "fn" => Token::Keyword(Keyword::Fn),
                "enum" => Token::Keyword(Keyword::Enum),
                "mod" => Token::Keyword(Keyword::Mod),
                "const" => Token::Keyword(Keyword::Const),
                "let" => Token::Keyword(Keyword::Let),
                "if" => Token::Keyword(Keyword::If),
                "else" => Token::Keyword(Keyword::Else),
                "for" => Token::Keyword(Keyword::For),
                "in" => Token::Keyword(Keyword::In),
                "while" => Token::Keyword(Keyword::While),
                "return" => Token::Keyword(Keyword::Return),
                "break" => Token::Keyword(Keyword::Break),
                "continue" => Token::Keyword(Keyword::Continue),
                "print" => Token::Keyword(Keyword::Print),
                _ => Token::Identifier(Ident { name: word }),
            };
            Ok(token)
        }

        fn lex_number(&mut self) -> Result<Token<'a>, LexError> {
            let idx = self
                .input
                .iter()
                .enumerate()
                .find_map(|(idx, &ch)| {
                    if !matches!(ch, b'-' | b'e' | b'E' | b'0'..=b'9' | b'.') {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .unwrap_or(self.input.len());
            let num = std::str::from_utf8(&self.input[..idx]).unwrap();
            self.input = &self.input[idx..];
            if let Ok(num) = num.parse::<i64>() {
                Ok(Token::Literal(Literal::Int(num)))
            } else if let Ok(num) = num.parse::<f64>() {
                Ok(Token::Literal(Literal::Float(num)))
            } else {
                Err(LexError {})
            }
        }

        fn lex_string(&mut self) -> Result<Token<'a>, LexError> {
            assert_eq!(self.input.first(), Some(&b'"'));
            self.input = &self.input[1..];
            let mut idx = 1;
            let mut escaped = false;
            while idx < self.input.len() {
                if self.input[idx] == b'"' {
                    break;
                } else if self.input[idx] == b'\\' {
                    idx += 1;
                    match self.input[idx] {
                        b'\\' | b'"' | b't' | b'n' | b'r' => escaped = true, // add more escape codes
                        _ => return Err(LexError {}),
                    }
                }
                idx += 1;
            }
            let word = std::str::from_utf8(&self.input[..idx]).unwrap();
            self.input = &self.input[idx + 1..]; // ignore the closing quote
            let token = if !escaped {
                Token::Literal(Literal::String(Cow::Borrowed(word)))
            } else {
                let mut s = String::with_capacity(3 * word.len() / 4);
                let mut chars = word.chars();
                while let Some(c) = chars.next() {
                    if c == '\\' {
                        let c = chars.next().unwrap();
                        match c {
                            '\\' => s.push('\\'),
                            '"' => s.push('"'),
                            't' => s.push('\t'),
                            'n' => s.push('\n'),
                            'r' => s.push('\r'),
                            _ => unreachable!(),
                        }
                        continue;
                    }
                    s.push(c);
                }
                Token::Literal(Literal::String(Cow::Owned(s)))
            };
            Ok(token)
        }
    }

    #[derive(Debug)]
    pub struct LexError {}

    #[derive(Debug)]
    pub enum Token<'a> {
        Literal(Literal<'a>),
        Symbol(Symbol),
        Identifier(Ident<'a>),
        Keyword(Keyword),
    }

    #[derive(Debug)]
    pub enum Literal<'a> {
        Int(i64),
        Float(f64),
        String(Cow<'a, str>),
        Char(char),
    }

    #[derive(Debug)]
    pub enum Symbol {
        DoubleColon,
        SemiColon,
        LeftBrace,
        RightBrace,
        LeftParam,
        RightParam,
        Plus,
        Minus,
        Cross,
        LeftSlash,
        RghtSlash,
        Colon,
        Dot,
        Comma,
        DoubleEqual,
        Equal,
        PlusEqual,
        MinusEqual,
    }

    #[derive(Debug)]
    pub struct Ident<'a> {
        name: &'a str,
    }

    #[derive(Debug)]
    pub enum Keyword {
        Import,
        Struct,
        Fn,
        Enum,
        Mod,
        Const,
        Let,
        If,
        Else,
        For,
        In,
        While,
        Return,
        Break,
        Continue,
        Print,
    }
}
