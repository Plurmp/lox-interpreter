use std::{borrow::Cow, fmt::Display, iter::Peekable};

use miette::{diagnostic, Context, Diagnostic, Error, LabeledSpan};

use crate::lex::{Lexer, Token, TokenKind};

#[derive(Debug)]
pub enum Expr<'a> {
    Atom(Atom<'a>),
    Cons(Op, Vec<Expr<'a>>),
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Atom(a) => write!(f, "{}", a),
            Expr::Cons(a, rest) => {
                write!(f, "({a}")?;
                for s in rest {
                    write!(f, " {s}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Atom<'a> {
    Number(f64),
    String(Cow<'a, str>),
    Bool(bool),
    Nil,
    Ident(&'a str),
}

impl Display for Atom<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n:?}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Ident(i) => write!(f, "{i}"),
            Self::Bool(b) => write!(f, "{b:?}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Op {
    Bang,
    Plus,
    Minus,
    Star,
    Slash,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Field,
    Group,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Bang => "!",
                Op::Plus => "+",
                Op::Minus => "-",
                Op::Star => "*",
                Op::Slash => "/",
                Op::EqualEqual => "==",
                Op::BangEqual => "!=",
                Op::Less => "<",
                Op::LessEqual => "<=",
                Op::Greater => ">",
                Op::GreaterEqual => ">=",
                Op::Field => ".",
                Op::Group => "group",
            }
        )
    }
}

#[derive(Debug)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl Display for Program<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(program")?;
        for s in &self.statements {
            write!(f, " {s}")?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub enum Statement<'a> {
    ExprStatement(Expr<'a>),
    PrintStatement(Expr<'a>),
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::ExprStatement(expr) => write!(f, "(exprStatement {expr})"),
            Statement::PrintStatement(expr) => write!(f, "(print {expr})"),
        }
    }
}

pub struct Parser<'a> {
    whole: &'a str,
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            whole: input,
            lexer: Lexer::new(input).peekable(),
        }
    }
    pub fn parse(&mut self) -> Result<Program<'a>, Error> {
        let mut statements = vec![];
        while let Some(token) = self.lexer.peek() {
            match token {
                Ok(Token {
                    kind: TokenKind::Print,
                    ..
                }) => {
                    self.lexer.next();
                    statements.push(Statement::PrintStatement(self.parse_expr_statement()?))
                }
                Ok(_) => statements.push(Statement::ExprStatement(self.parse_expr_statement()?)),
                Err(_) => {
                    return Err(match self.lexer.next().unwrap() {
                        Err(e) => e,
                        Ok(_) => unreachable!("peek was already Err"),
                    })
                }
            }
        }

        Ok(Program { statements })
    }
    fn parse_expr_statement(&mut self) -> Result<Expr<'a>, Error> {
        let expr = self.expr()?;
        match self.lexer.next().transpose()? {
            Some(Token {
                kind: TokenKind::Semicolon,
                ..
            }) => Ok(expr),
            Some(t) => Err(diagnostic!("Expected a semicolon at {}", t.origin).into()),
            None => Err(diagnostic!("Unexpected EOF").into()),
        }
    }
    pub fn expr(&mut self) -> Result<Expr<'a>, Error> {
        self.expr_bp(0)
    }
    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr<'a>, Error> {
        let lhs = match self.lexer.next() {
            Some(Ok(token)) => token,
            None => return Ok(Expr::Atom(Atom::Nil)),
            Some(Err(e)) => {
                return Err(e).wrap_err("on left hand side");
            }
        };

        let mut lhs = match lhs {
            Token {
                kind: TokenKind::Number(n),
                ..
            } => Expr::Atom(Atom::Number(n)),
            Token {
                kind: TokenKind::Ident,
                origin,
                ..
            } => Expr::Atom(Atom::Ident(origin)),
            Token {
                kind: TokenKind::Bang | TokenKind::Minus,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::Bang => Op::Bang,
                    TokenKind::Minus => Op::Minus,
                    _ => unreachable!("by outer match arm"),
                };
                let ((), r_bp) = prefix_binding_power(op);
                let rhs = self.expr_bp(r_bp)?;
                Expr::Cons(op, vec![rhs])
            }
            Token {
                kind: TokenKind::LeftParen,
                ..
            } => {
                let lhs = self.expr_bp(0).wrap_err("in parentheses")?;
                let Some(Ok(Token {
                    kind: TokenKind::RightParen,
                    ..
                })) = self.lexer.next()
                else {
                    return Err(diagnostic!("expected right parentheses").into());
                };

                Expr::Cons(Op::Group, vec![lhs])
            }
            Token {
                kind: TokenKind::String,
                origin,
                ..
            } => Expr::Atom(Atom::String(origin.trim_matches('"').into())),
            token => {
                return Err(diagnostic!(
                    labels = vec![LabeledSpan::at(
                        token.start..token.start + token.origin.len(),
                        "here"
                    )],
                    "Expected a statement"
                )
                .into())
            }
        };

        loop {
            let op = self.lexer.peek();

            if op.map_or(false, |op| op.is_err()) {
                return Err(self
                    .lexer
                    .next()
                    .expect("Checked Some above")
                    .expect_err("checked Err above"))
                .wrap_err("in place of expected Op");
            }

            let op = match op.map(|res| res.as_ref().expect("handled Err above")) {
                None
                | Some(Token {
                    kind: TokenKind::RightParen | TokenKind::Semicolon,
                    ..
                }) => break,
                Some(Token {
                    kind: TokenKind::Plus,
                    ..
                }) => Op::Plus,
                Some(Token {
                    kind: TokenKind::Minus,
                    ..
                }) => Op::Minus,
                Some(Token {
                    kind: TokenKind::Star,
                    ..
                }) => Op::Star,
                Some(Token {
                    kind: TokenKind::Slash,
                    ..
                }) => Op::Slash,
                Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) => Op::Field,
                Some(Token {
                    kind: TokenKind::Greater,
                    ..
                }) => Op::Greater,
                Some(Token {
                    kind: TokenKind::GreaterEqual,
                    ..
                }) => Op::GreaterEqual,
                Some(Token {
                    kind: TokenKind::Less,
                    ..
                }) => Op::Less,
                Some(Token {
                    kind: TokenKind::LessEqual,
                    ..
                }) => Op::LessEqual,
                Some(Token {
                    kind: TokenKind::BangEqual,
                    ..
                }) => Op::BangEqual,
                Some(Token {
                    kind: TokenKind::EqualEqual,
                    ..
                }) => Op::EqualEqual,
                Some(token) => {
                    return Err(diagnostic!(
                        labels = vec![LabeledSpan::at(
                            token.start..token.start + token.origin.len(),
                            "here"
                        )],
                        "Expected an operator, got {} instead",
                        token
                    )
                    .into());
                }
            };

            if let Some((l_bp, ())) = postfix_binding_power(op) {
                if l_bp < min_bp {
                    break;
                }
                self.lexer.next();

                lhs = Expr::Cons(op, vec![lhs]);
                continue;
            }

            if let Some((l_bp, r_bp)) = infix_binding_power(op) {
                if l_bp < min_bp {
                    break;
                }
                self.lexer.next();
                let rhs = self
                    .expr_bp(r_bp)
                    .wrap_err_with(|| format!("on rhs of {lhs} {op}"))?;
                lhs = Expr::Cons(op, vec![lhs, rhs]);
                continue;
            }

            break;
        }

        Ok(lhs)
    }
}

fn infix_binding_power(op: Op) -> Option<(u8, u8)> {
    Some(match op {
        Op::Plus | Op::Minus => (5, 6),
        Op::Star | Op::Slash => (7, 8),
        Op::EqualEqual | Op::BangEqual => (1, 2),
        Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual => (3, 4),
        Op::Field => (11, 10),
        _ => return None,
    })
}

fn prefix_binding_power(op: Op) -> ((), u8) {
    match op {
        Op::Plus | Op::Minus => ((), 9),
        _ => panic!("unexpected operator {op}"),
    }
}

fn postfix_binding_power(op: Op) -> Option<(u8, ())> {
    Some(match op {
        _ => return None,
    })
}

macro_rules! expr_test {
    ($expression:literal, $s_str:literal) => {{
        let s = Parser::new($expression).expr().unwrap();
        assert_eq!(s.to_string(), $s_str);
    }};
}

#[test]
fn one() {
    let s = Parser::new("1").expr().unwrap();
    assert_eq!(s.to_string(), "1.0");
}

#[test]
fn add_and_mult() {
    let s = Parser::new("1 + 2 * 3").expr().unwrap();
    assert_eq!(s.to_string(), "(+ 1.0 (* 2.0 3.0))");

    let s = Parser::new("a + b * c * d + e").expr().unwrap();
    assert_eq!(s.to_string(), "(+ (+ a (* (* b c) d)) e)");
}

#[test]
fn field() {
    let s = Parser::new("foo.bar").expr().unwrap();
    assert_eq!(s.to_string(), "(. foo bar)");

    let s = Parser::new("1 + 2 + f.g.h * 3 * 4").expr().unwrap();
    assert_eq!(
        s.to_string(),
        "(+ (+ 1.0 2.0) (* (* (. f (. g h)) 3.0) 4.0))"
    )
}

#[test]
fn prefix() {
    expr_test!("--1 * 2", "(* (- (- 1.0)) 2.0)");
    expr_test!("--f.g", "(- (- (. f g)))");
}

#[test]
fn groups() {
    expr_test!("(((0)))", "(group (group (group 0.0)))");
    expr_test!(
        "(5 - (3 - 1)) + -1",
        "(+ (group (- 5.0 (group (- 3.0 1.0)))) (- 1.0))"
    )
}

#[test]
fn string() {
    expr_test!(r#""Hello World""#, "Hello World");
}

#[test]
fn print() {
    let program = Parser::new(r#"print "Hello World";"#).parse().unwrap();
    assert_eq!(program.to_string(), r#"(program (print Hello World))"#);
}
