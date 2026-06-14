use crate::parse::{Atom, Expr, Op};

fn eval_expr<'a>(s: &'a Expr) -> Atom<'a> {
    match s {
        Expr::Atom(atom) => atom.clone(),
        Expr::Cons(op, rest) => {
            if rest.len() == 2 {
                let first = eval_expr(&rest[0]);
                let second = eval_expr(&rest[1]);

                match op {
                    Op::Plus => {
                        if let Atom::Number(first) = first && let Atom::Number(second) = second {
                            Atom::Number(first + second)
                        } else {
                            todo!();
                        }
                    }
                    Op::Star => {
                        if let Atom::Number(first) = first && let Atom::Number(second) = second {
                            Atom::Number(first * second)
                        } else {
                            todo!();
                        }
                    }
                    Op::Minus => {
                        if let Atom::Number(first) = first && let Atom::Number(second) = second {
                            Atom::Number(first - second)
                        } else {
                            todo!();
                        }
                    }
                    Op::Slash => {
                        if let Atom::Number(first) = first && let Atom::Number(second) = second {
                            Atom::Number(first / second)
                        } else {
                            todo!();
                        }
                    }
                    _ => todo!(),
                }
            } else if rest.len() == 1 {
                let first = eval_expr(&rest[0]);
                
                match op {
                    Op::Group => {
                        first
                    }
                    _ => todo!(),
                }
            } else {
                todo!()
            }
        }
    }
}

macro_rules! test_eval {
    ($expression:literal, $final:expr) => {
        let expr = crate::parse::Parser::new($expression).expr().unwrap();
        println!("{expr}");
        assert_eq!(eval_expr(&expr), $final);
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
