use std::borrow::Cow;

pub struct Lexer<'a> {
    cur_line: usize,
    cur_col: usize,
    input: &'a [u8],
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            cur_line: 1,
            cur_col: 1,
            input,
            tokens: vec![],
        }
    }

    pub fn get_tokens(self) -> Vec<Token<'a>> {
        self.tokens
    }

    pub fn lex(&mut self) -> Result<(), LexError> {
        while let Some(ch) = self.peek() {
            match ch {
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                    let token = self.lex_word()?;
                    self.tokens.push(token);
                }
                b'0'..=b'9' => {
                    let token = self.lex_number()?;
                    self.tokens.push(token);
                }
                b'"' => {
                    let token = self.lex_string()?;
                    self.tokens.push(token);
                }
                b'\'' => {
                    let token = self.lex_character()?;
                    self.tokens.push(token);
                }
                b'\n' => {
                    self.cur_line += 1;
                    self.cur_col = 1;
                    self.advance(1);
                }
                c if c.is_ascii_whitespace() => {
                    self.cur_col += 1;
                    self.advance(1);
                }
                _ => {
                    let token = self.try_lex_symbol()?;
                    self.tokens.push(token);
                }
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
        self.advance(idx);
        let pos = Position {
            line: self.cur_line,
            col: self.cur_col,
        };
        self.cur_col += idx;
        let token = match word {
            "import" => Kind::Keyword(Keyword::Import),
            "struct" => Kind::Keyword(Keyword::Struct),
            "fn" => Kind::Keyword(Keyword::Fn),
            "enum" => Kind::Keyword(Keyword::Enum),
            "mod" => Kind::Keyword(Keyword::Mod),
            "const" => Kind::Keyword(Keyword::Const),
            "let" => Kind::Keyword(Keyword::Let),
            "if" => Kind::Keyword(Keyword::If),
            "else" => Kind::Keyword(Keyword::Else),
            "for" => Kind::Keyword(Keyword::For),
            "in" => Kind::Keyword(Keyword::In),
            "while" => Kind::Keyword(Keyword::While),
            "return" => Kind::Keyword(Keyword::Return),
            "break" => Kind::Keyword(Keyword::Break),
            "continue" => Kind::Keyword(Keyword::Continue),
            "print" => Kind::Keyword(Keyword::Print),
            _ => Kind::Identifier(Ident { name: word }),
        };
        let token = Token { pos, kind: token };
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
        self.advance(idx);
        let pos = Position {
            line: self.cur_line,
            col: self.cur_col,
        };
        self.cur_col += idx;
        let token = if let Ok(num) = num.parse::<i64>() {
            Kind::Literal(Literal::Int(num))
        } else if let Ok(num) = num.parse::<f64>() {
            Kind::Literal(Literal::Float(num))
        } else {
            return Err(LexError {});
        };
        let token = Token { pos, kind: token };
        Ok(token)
    }

    fn lex_string(&mut self) -> Result<Token<'a>, LexError> {
        assert_eq!(self.peek(), Some(b'"'));
        self.advance(1);
        let mut idx = 0;
        let mut newlines = 0;
        let mut new_col = self.cur_col + 1;
        let mut escaped = false;
        while idx < self.input.len() {
            if self.input[idx] == b'"' {
                new_col += 1;
                break;
            } else if self.input[idx] == b'\\' {
                idx += 1;
                new_col += 1;
                match self.input[idx] {
                    b'\\' | b'"' | b't' | b'n' | b'r' => escaped = true, // add more escape codes
                    _ => return Err(LexError {}),
                }
            } else if self.input[idx] == b'\n' {
                newlines += 1;
                new_col = 0;
            }
            idx += 1;
            new_col += 1;
        }
        let word = std::str::from_utf8(&self.input[..idx]).unwrap();
        if idx == self.input.len() {
            return Err(LexError {});
        }
        self.input = &self.input[idx + 1..]; // ignore the closing quote
        let pos = Position {
            line: self.cur_line,
            col: self.cur_col,
        };
        self.cur_col = new_col;
        self.cur_line += newlines; // multiline strings are allowed

        let token = if !escaped {
            Kind::Literal(Literal::String(Cow::Borrowed(word)))
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
            Kind::Literal(Literal::String(Cow::Owned(s)))
        };
        let token = Token { pos, kind: token };
        Ok(token)
    }

    fn lex_character(&mut self) -> Result<Token<'a>, LexError> {
        assert_eq!(self.peek(), Some(b'\''));
        let pos = Position {
            line: self.cur_line,
            col: self.cur_col,
        };
        self.advance(1);
        self.cur_col += 3; // "'" character "'"
        let character = if let Some(b'\\') = self.peek() {
            self.advance(1);
            self.cur_col += 1;
            match self.peek() {
                Some(b'\'') => '\'',
                Some(b'\\') => '\\',
                Some(b't') => '\t',
                Some(b'n') => '\n',
                Some(b'r') => '\r',
                _ => return Err(LexError {}),
            }
        } else if let Some(c) = self.peek() {
            c as char
        } else {
            return Err(LexError {});
        };
        self.advance(1);
        if let Some(b'\'') = self.peek() {
            self.advance(1);
        } else {
            return Err(LexError {});
        }
        let token = Kind::Literal(Literal::Char(character));
        let token = Token { pos, kind: token };
        Ok(token)
    }

    fn try_lex_symbol(&mut self) -> Result<Token<'a>, LexError> {
        let pos = Position {
            col: self.cur_col,
            line: self.cur_line,
        };
        let ch = self.input[0];
        let token = match ch {
            b'[' => Kind::Symbol(Symbol::LeftSquareBracket),
            b']' => Kind::Symbol(Symbol::RightSquareBracket),
            b'(' => Kind::Symbol(Symbol::LeftParam),
            b')' => Kind::Symbol(Symbol::RightParam),
            b'{' => Kind::Symbol(Symbol::LeftBrace),
            b'}' => Kind::Symbol(Symbol::RightBrace),
            b';' => Kind::Symbol(Symbol::SemiColon),
            b'.' => Kind::Symbol(Symbol::Dot),
            b',' => Kind::Symbol(Symbol::Comma),
            b':' => {
                if let Some(b':') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::DoubleColon)
                } else {
                    Kind::Symbol(Symbol::Colon)
                }
            }
            b'+' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::PlusEqual)
                } else {
                    Kind::Symbol(Symbol::Plus)
                }
            }
            b'-' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::MinusEqual)
                } else if let Some(b'>') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::Arrow)
                } else {
                    Kind::Symbol(Symbol::Minus)
                }
            }
            b'*' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::StarEqual)
                } else {
                    Kind::Symbol(Symbol::Star)
                }
            }
            b'/' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::SlashEqual)
                } else {
                    Kind::Symbol(Symbol::Slash)
                }
            }
            b'>' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::GreaterThanEqual)
                } else if let Some(b'>') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    if let Some(b'=') = self.peek_next() {
                        self.advance(1);
                        self.cur_col += 1;
                        Kind::Symbol(Symbol::ShrEqual)
                    } else {
                        Kind::Symbol(Symbol::Shr)
                    }
                } else {
                    Kind::Symbol(Symbol::GreaterThan)
                }
            }
            b'<' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::LessThanEqual)
                } else if let Some(b'<') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    if let Some(b'=') = self.peek_next() {
                        self.advance(1);
                        self.cur_col += 1;
                        Kind::Symbol(Symbol::ShlEqual)
                    } else {
                        Kind::Symbol(Symbol::Shl)
                    }
                } else {
                    Kind::Symbol(Symbol::LessThan)
                }
            }
            b'|' => {
                if let Some(b'|') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::DoublePipe)
                } else if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::PipeEqual)
                } else {
                    Kind::Symbol(Symbol::Pipe)
                }
            }
            b'&' => {
                if let Some(b'&') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::DoubleAmpersand)
                } else if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::AmpersandEqual)
                } else {
                    Kind::Symbol(Symbol::Ampersand)
                }
            }
            b'^' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::CaretEqual)
                } else {
                    Kind::Symbol(Symbol::Caret)
                }
            }
            b'=' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::DoubleEqual)
                } else if let Some(b'>') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::FatArrow)
                } else {
                    Kind::Symbol(Symbol::Equal)
                }
            }
            b'\\' => Kind::Symbol(Symbol::BackSlash),
            b'!' => {
                if let Some(b'=') = self.peek_next() {
                    self.advance(1);
                    self.cur_col += 1;
                    Kind::Symbol(Symbol::BangEqual)
                } else {
                    Kind::Symbol(Symbol::Bang)
                }
            }
            _ => return Err(LexError {}),
        };
        self.cur_col += 1;
        self.advance(1);
        Ok(Token { pos, kind: token })
    }

    fn advance(&mut self, by: usize) {
        self.input = &self.input[by..];
    }

    fn peek(&self) -> Option<u8> {
        self.input.first().copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.input.get(1).copied()
    }
}

#[derive(Debug)]
pub struct LexError {}

#[derive(Debug)]
pub struct Position {
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct Token<'a> {
    pos: Position,
    kind: Kind<'a>,
}

#[derive(Debug)]
pub enum Kind<'a> {
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
    Colon,              // :
    DoubleColon,        // ::
    SemiColon,          // ;
    LeftBrace,          // {
    RightBrace,         // }
    LeftParam,          // (
    RightParam,         // )
    LeftSquareBracket,  // [
    RightSquareBracket, // ]
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    PlusEqual,          // +=
    MinusEqual,         // -=
    StarEqual,          // *=
    SlashEqual,         // /=
    Pipe,               // |
    DoublePipe,         // ||
    Ampersand,          // &
    DoubleAmpersand,    // &&
    Caret,              // ^
    CaretEqual,         // ^=
    PipeEqual,          // |=
    AmpersandEqual,     // &=
    LessThan,           // <
    GreaterThan,        // >
    LessThanEqual,      // <=
    GreaterThanEqual,   // >=
    Shr,                // >>
    Shl,                // <<
    ShrEqual,           // >>=
    ShlEqual,           // <<=
    Bang,               // !
    BangEqual,          // !=
    Dot,                // .
    BackSlash,          // \
    Comma,              // ,
    DoubleEqual,        // ==
    Equal,              // =
    Arrow,              // ->
    FatArrow,           // =>
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
