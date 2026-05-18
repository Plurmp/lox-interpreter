use std::fmt::Display;

use miette::SourceOffset;

#[derive(Debug)]
pub enum S<'a> {
    Atom(Atom<'a>),
    Cons(Atom<'a>, Vec<S<'a>>),
}

impl Display for S<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S::Atom(a) => write!(f, "{}", a),
            S::Cons(a, rest) => {
                write!(f, "({}", a)?;
                for s in rest {
                    write!(f, " {s}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug)]
pub enum Atom<'a> {
    Number(f64),
    String(&'a str),
    Bool(bool),
    Nil,
}

impl Display for Atom<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n:?}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Bool(b) => write!(f, "{b:?}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug)]
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
            }
        )
    }
}
