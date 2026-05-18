use std::fmt::Display;

use miette::{diagnostic, Error, LabeledSpan};

#[derive(Debug)]
pub struct Token<'de> {
    kind: TokenKind,
    origin: &'de str,
    start: usize,
}

#[derive(Debug)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    Plus,
    Minus,
    Star,
    Bang,
    Equal,
    EqualEqual,
    LessEqual,
    GreaterEqual,
    BangEqual,
    Less,
    Greater,
    Slash,
    Dot,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    String,
    Number(f64),
    Ident,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let origin = self.origin;
        use TokenKind as TK;
        match self.kind {
            TK::LeftParen => write!(f, "LEFT_PAREN {origin} null"),
            TK::RightParen => write!(f, "RIGHT_PAREN {origin} null"),
            TK::LeftBrace => write!(f, "LEFT_BRACE {origin} null"),
            TK::RightBrace => write!(f, "RIGHT_BRACE {origin} null"),
            TK::Semicolon => write!(f, "SEMICOLON {origin} null"),
            TK::Comma => write!(f, "COMMA {origin} null"),
            TK::Plus => write!(f, "PLUS {origin} null"),
            TK::Minus => write!(f, "MINUS {origin} null"),
            TK::Star => write!(f, "STAR {origin} null"),
            TK::Bang => write!(f, "BANG {origin} null"),
            TK::Equal => write!(f, "EQUAL {origin} null"),
            TK::EqualEqual => write!(f, "EQUAL_EQUAL {origin} null"),
            TK::LessEqual => write!(f, "LESS_EQUAL {origin} null"),
            TK::GreaterEqual => write!(f, "GREATER_EQUAL {origin} null"),
            TK::BangEqual => write!(f, "BANG_EQUAL {origin} null"),
            TK::Less => write!(f, "LESS {origin} null"),
            TK::Greater => write!(f, "GREATER {origin} null"),
            TK::Slash => write!(f, "SLASH {origin} null"),
            TK::Dot => write!(f, "DOT {origin} null"),
            TK::And => write!(f, "AND {origin} null"),
            TK::Class => write!(f, "CLASS {origin} null"),
            TK::Else => write!(f, "ELSE {origin} null"),
            TK::False => write!(f, "FALSE {origin} null"),
            TK::For => write!(f, "FOR {origin} null"),
            TK::Fun => write!(f, "FUN {origin} null"),
            TK::If => write!(f, "IF {origin} null"),
            TK::Nil => write!(f, "NIL {origin} null"),
            TK::Or => write!(f, "OR {origin} null"),
            TK::Print => write!(f, "PRINT {origin} null"),
            TK::Return => write!(f, "RETURN {origin} null"),
            TK::Super => write!(f, "SUPER {origin} null"),
            TK::This => write!(f, "THIS {origin} null"),
            TK::True => write!(f, "TRUE {origin} null"),
            TK::Var => write!(f, "VAR {origin} null"),
            TK::While => write!(f, "WHILE {origin} null"),
            TK::String => write!(f, "STRING {origin} {}", origin.trim_matches('"')),
            TK::Number(n) => write!(f, "NUMBER {origin} {:?}", n),
            TK::Ident => write!(f, "IDENTIFIER {origin} null"),
        }
    }
}

pub struct Lexer<'de> {
    whole: &'de str,
    rest: &'de str,
    byte: usize,
}

impl<'de> Lexer<'de> {
    fn new(input: &'de str) -> Self {
        Lexer {
            whole: input,
            rest: input,
            byte: 0,
        }
    }
}

impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut chars = self.rest.chars();
            let c = chars.next()?;
            let c_byte = self.byte;
            let c_str = &self.rest[..c.len_utf8()];
            let c_rest = self.rest;
            self.rest = chars.as_str();
            self.byte += c.len_utf8();

            let just = |kind: TokenKind| {
                Some(Ok(Token {
                    kind,
                    origin: c_str,
                    start: c_byte,
                }))
            };

            enum Started {
                IfEqualElse(TokenKind, TokenKind),
                Slash,
                String,
                Number,
                Ident,
            }

            use TokenKind as TK;
            let started = match c {
                '(' => return just(TK::LeftParen),
                ')' => return just(TK::RightParen),
                '{' => return just(TK::LeftBrace),
                '}' => return just(TK::RightBrace),
                ',' => return just(TK::Comma),
                '+' => return just(TK::Plus),
                '-' => return just(TK::Minus),
                '*' => return just(TK::Star),
                ';' => return just(TK::Semicolon),
                '.' => return just(TK::Dot),
                '<' => Started::IfEqualElse(TK::LessEqual, TK::Less),
                '>' => Started::IfEqualElse(TK::GreaterEqual, TK::Greater),
                '!' => Started::IfEqualElse(TK::BangEqual, TK::Bang),
                '=' => Started::IfEqualElse(TK::EqualEqual, TK::Equal),
                '/' => Started::Slash,
                '"' => Started::String,
                '0'..='9' => Started::Number,
                'A'..='Z' | 'a'..='z' | '_' => Started::Ident,
                c if c.is_whitespace() => continue,
                _ => {
                    return Some(Err(diagnostic! {
                        labels = vec![LabeledSpan::at_offset(c_byte, "this character")],
                        "unrecognized character {c}",
                    }
                    .into()))
                }
            };

            break match started {
                Started::IfEqualElse(left, right) => {
                    self.rest = self.rest.trim_start();
                    let trimmed = c_rest.len() - self.rest.len() - 1;
                    self.byte += trimmed;
                    if self.rest.starts_with('=') {
                        let span = &c_rest[..c.len_utf8() + trimmed + 1];
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        Some(Ok(Token {
                            kind: left,
                            origin: span,
                            start: c_byte,
                        }))
                    } else {
                        Some(Ok(Token {
                            kind: right,
                            origin: c_str,
                            start: c_byte,
                        }))
                    }
                }
                Started::Slash => {
                    if self.rest.starts_with('/') {
                        let line_end = self.rest.find('\n').unwrap_or_else(|| self.rest.len());
                        self.byte += line_end;
                        self.rest = &self.rest[line_end..];
                        continue;
                    } else {
                        Some(Ok(Token {
                            origin: c_str,
                            start: c_byte,
                            kind: TokenKind::Slash,
                        }))
                    }
                }
                Started::String => {
                    if let Some(end) = self.rest.find('"') {
                        let literal = &c_rest[..end + 2];
                        self.byte += end + 1;
                        self.rest = &self.rest[end + 1..];
                        Some(Ok(Token {
                            origin: literal,
                            start: c_byte,
                            kind: TokenKind::String,
                        }))
                    } else {
                        let err = diagnostic!(
                            labels = vec![LabeledSpan::at(
                                self.byte - c.len_utf8()..self.whole.len(),
                                "here"
                            )],
                            "unclosed string literal"
                        );

                        self.byte += self.rest.len();
                        self.rest = &self.rest[self.rest.len()..];

                        return Some(Err(err.into()));
                    }
                }
                Started::Number => {
                    let first_non_digit = c_rest
                        .find(|c| !matches!(c, '.' | '0'..='9'))
                        .unwrap_or_else(|| c_rest.len());
                    let mut literal = &c_rest[..first_non_digit];
                    let mut parts = literal.splitn(3, '.');
                    match (parts.next(), parts.next(), parts.next()) {
                        (Some(first), Some(second), Some(_)) => {
                            literal = &literal[..first.len() + 1 + second.len()];
                        }
                        (Some(first), Some(second), None) if second.is_empty() => {
                            literal = &literal[..first.len()];
                        }
                        _ => {}
                    }

                    let extra_bytes = literal.len() - c.len_utf8();
                    self.byte += extra_bytes;
                    self.rest = &self.rest[extra_bytes..];

                    let n = match literal.parse() {
                        Ok(n) => n,
                        Err(e) => {
                            return Some(Err(diagnostic!(
                                labels =
                                    vec![LabeledSpan::at(c_byte..c_byte + literal.len(), "here",)],
                                "unable to parse number: {e}"
                            )
                            .into()))
                        }
                    };

                    Some(Ok(Token {
                        origin: literal,
                        start: c_byte,
                        kind: TokenKind::Number(n),
                    }))
                }
                Started::Ident => {
                    let first_non_ident = c_rest
                        .find(|c| !matches!(c, 'A'..='Z' | 'a'..='z' | '_' | '0'..='9'))
                        .unwrap_or_else(|| c_rest.len());
                    let ident = &c_rest[..first_non_ident];
                    let extra_bytes = ident.len() - c.len_utf8();
                    self.byte += extra_bytes;
                    self.rest = &self.rest[extra_bytes..];

                    use TokenKind as TK;
                    let kind = match ident {
                        "and" => TK::And,
                        "class" => TK::Class,
                        "else" => TK::Else,
                        "false" => TK::False,
                        "for" => TK::For,
                        "fun" => TK::Fun,
                        "if" => TK::If,
                        "nil" => TK::Nil,
                        "or" => TK::Or,
                        "print" => TK::Print,
                        "return" => TK::Return,
                        "super" => TK::Super,
                        "this" => TK::This,
                        "true" => TK::True,
                        "var" => TK::Var,
                        "while" => TK::While,
                        _ => TK::Ident,
                    };

                    return Some(Ok(Token {
                        origin: ident,
                        start: c_byte,
                        kind,
                    }));
                }
            };
        }
    }
}

#[test]
fn one_character() {
    assert_eq!(
        Lexer::new("(){};,+-*.")
            .map(|token| -> String {
                match token {
                    Ok(t) => t.to_string(),
                    Err(_) => panic!(),
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "LEFT_PAREN ( null
RIGHT_PAREN ) null
LEFT_BRACE { null
RIGHT_BRACE } null
SEMICOLON ; null
COMMA , null
PLUS + null
MINUS - null
STAR * null
DOT . null",
    );
}

#[test]
fn equals() {
    assert_eq!(
        Lexer::new("<= >= == = < > ! !=")
            .map(|token| -> String {
                match token {
                    Ok(t) => t.to_string(),
                    Err(e) => panic!("{e}"),
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "LESS_EQUAL <= null
GREATER_EQUAL >= null
EQUAL_EQUAL == null
EQUAL = null
LESS < null
GREATER > null
BANG ! null
BANG_EQUAL != null",
    )
}

#[test]
fn punctuators() -> Result<(), Error> {
    let result = Lexer::new("(){};,+-*!===<=>=!=<>/.")
        .map(|token| -> String {
            match token {
                Ok(t) => t.to_string(),
                Err(e) => panic!("{e}"),
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        result,
        "LEFT_PAREN ( null
RIGHT_PAREN ) null
LEFT_BRACE { null
RIGHT_BRACE } null
SEMICOLON ; null
COMMA , null
PLUS + null
MINUS - null
STAR * null
BANG_EQUAL != null
EQUAL_EQUAL == null
LESS_EQUAL <= null
GREATER_EQUAL >= null
BANG_EQUAL != null
LESS < null
GREATER > null
SLASH / null
DOT . null"
    );

    Ok(())
}

#[test]
fn punctuators2() {
    assert_eq!(
        Lexer::new("(){};,+-*!===<=>=!=<>/.")
            .map(|token| -> String {
                match token {
                    Ok(t) => t.to_string(),
                    Err(e) => panic!("{e}"),
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "LEFT_PAREN ( null
RIGHT_PAREN ) null
LEFT_BRACE { null
RIGHT_BRACE } null
SEMICOLON ; null
COMMA , null
PLUS + null
MINUS - null
STAR * null
BANG_EQUAL != null
EQUAL_EQUAL == null
LESS_EQUAL <= null
GREATER_EQUAL >= null
BANG_EQUAL != null
LESS < null
GREATER > null
SLASH / null
DOT . null"
    )
}

#[test]
fn strings() {
    assert_eq!(
        Lexer::new(
            "\"\"
\"string\""
        )
        .map(|token| -> String {
            match token {
                Ok(t) => t.to_string(),
                Err(e) => panic!("{e}"),
            }
        })
        .collect::<Vec<_>>()
        .join("\n"),
        "STRING \"\" 
STRING \"string\" string"
    )
}

#[test]
fn numbers() {
    assert_eq!(
        Lexer::new(
            "123
123.456
.456
123."
        )
        .map(|token| -> String {
            match token {
                Ok(t) => t.to_string(),
                Err(e) => panic!("{e}"),
            }
        })
        .collect::<Vec<_>>()
        .join("\n"),
        "NUMBER 123 123.0
NUMBER 123.456 123.456
DOT . null
NUMBER 456 456.0
NUMBER 123 123.0
DOT . null"
    )
}

#[test]
fn keywords() {
    assert_eq!(
        Lexer::new("and class else false for fun if nil or return super this true var while")
            .map(|token| -> String {
                match token {
                    Ok(t) => t.to_string(),
                    Err(e) => panic!("{e}"),
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "AND and null
CLASS class null
ELSE else null
FALSE false null
FOR for null
FUN fun null
IF if null
NIL nil null
OR or null
RETURN return null
SUPER super null
THIS this null
TRUE true null
VAR var null
WHILE while null"
    )
}

#[test]
fn identifiers() {
    assert_eq!(
        Lexer::new(
            "andy formless fo _ _123 _abc ab123
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_"
        )
        .map(|token| -> String {
            match token {
                Ok(t) => t.to_string(),
                Err(e) => panic!("{e}"),
            }
        })
        .collect::<Vec<_>>()
        .join("\n"),
        "IDENTIFIER andy null
IDENTIFIER formless null
IDENTIFIER fo null
IDENTIFIER _ null
IDENTIFIER _123 null
IDENTIFIER _abc null
IDENTIFIER ab123 null
IDENTIFIER abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_ null"
    )
}
