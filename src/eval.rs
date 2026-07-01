use crate::{
    parse::{Atom, Expr, Op},
    run::Environment,
};

use miette::{diagnostic, Error};

pub fn eval_expr<'a>(s: &'a Expr, env: &mut Environment<'a>) -> Result<Atom<'a>, Error> {
    match s {
        Expr::Atom(Atom::Ident(ident)) => Ok(env.get(ident)?.clone()),
        Expr::Atom(atom) => Ok(atom.clone()),
        Expr::Cons(op, rest) => {
            let first = rest.get(0).map(|f| eval_expr(f, env)).transpose()?;
            let second = rest.get(1).map(|s| eval_expr(s, env)).transpose()?;
            match (op, first, second) {
                (Op::Group, Some(first), None) => Ok(first),
                (Op::Plus, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Number(first + second))
                }
                (Op::Minus, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Number(first - second))
                }
                (Op::Star, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Number(first * second))
                }
                (Op::Slash, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Number(first / second))
                }
                (Op::Minus, Some(Atom::Number(first)), None) => Ok(Atom::Number(-first)),
                (Op::Bang, Some(first), None) => Ok(Atom::Bool(!is_truthy(&first))),
                (Op::Plus, Some(Atom::String(mut first)), Some(Atom::String(second))) => {
                    first.to_mut().push_str(&second);
                    Ok(Atom::String(first))
                }
                (Op::Greater, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Bool(first > second))
                }
                (Op::GreaterEqual, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Bool(first >= second))
                }
                (Op::Less, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Bool(first < second))
                }
                (Op::LessEqual, Some(Atom::Number(first)), Some(Atom::Number(second))) => {
                    Ok(Atom::Bool(first <= second))
                }
                (Op::BangEqual, Some(first), Some(second)) => Ok(Atom::Bool(first != second)),
                (Op::EqualEqual, Some(first), Some(second)) => Ok(Atom::Bool(first == second)),
                _ => Err(diagnostic!("malformed expression {s:?}").into()),
            }
        }
    }
}

fn is_truthy(atom: &Atom) -> bool {
    match atom {
        Atom::Bool(b) => *b,
        Atom::Nil => false,
        _ => true,
    }
}

#[allow(unused)]
macro_rules! test_eval {
    ($expression:literal, $final:expr) => {
        let mut env = Environment::new();
        let expr = crate::parse::Parser::new($expression).expr().unwrap();
        println!("{expr}");
        assert_eq!(
            eval_expr(&expr, &mut env).expect("Evaluation failed"),
            $final
        );
    };
}

#[test]
fn add_num() {
    test_eval!("1 + 1", Atom::Number(2.0));
    test_eval!("1 + (1 + 1)", Atom::Number(3.0));
}

#[test]
fn mult_num() {
    test_eval!("1 * 2", Atom::Number(2.0));
    test_eval!("3 * 4 * (2 * 2)", Atom::Number(48.0));
}

#[test]
fn sub_num() {
    test_eval!("2 - 1", Atom::Number(1.0));
    test_eval!("4 - (5 - 2)", Atom::Number(1.0));
}

#[test]
fn div_num() {
    test_eval!("24 / 6", Atom::Number(4.0));
    test_eval!("50 / (20 / 2)", Atom::Number(5.0));
}

#[test]
fn add_string() {
    test_eval!(r#""foo" + "bar""#, Atom::String("foobar".into()));
}

#[test]
fn comparison() {
    test_eval!("4 > 5", Atom::Bool(false));
    test_eval!("2 <= 2", Atom::Bool(true));
    test_eval!("5 == 5", Atom::Bool(true));
    test_eval!(r#""foo" == "foo""#, Atom::Bool(true));
}
