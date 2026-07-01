use std::collections::HashMap;

use miette::{miette, IntoDiagnostic};

use crate::{
    eval::eval_expr,
    parse::{Atom, Expr, Program, Statement, VarDecl},
    Parser,
};

pub struct Environment<'a> {
    values: HashMap<&'a str, Atom<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &'a str, value: Atom<'a>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> miette::Result<&Atom<'a>> {
        if let Some(value) = self.values.get(name) {
            Ok(value)
        } else {
            Err(miette!("Undefined variable {name}"))
        }
    }
}

pub fn run<W: std::io::Write>(program: &Program, stdout: &mut W) -> miette::Result<()> {
    let mut env = Environment::new();
    for statement in &program.statements {
        match statement {
            Statement::PrintStatement(expr) => visit_print_statement(expr, stdout, &mut env)?,
            Statement::VarDecl(var_decl) => visit_var_decl(var_decl, &mut env)?,
            _ => todo!(),
        }
    }

    Ok(())
}

fn visit_print_statement<'a, W: std::io::Write>(
    expr: &'a Expr,
    stdout: &mut W,
    env: &mut Environment<'a>,
) -> miette::Result<()> {
    write!(stdout, "{}", eval_expr(expr, env)?).into_diagnostic()?;

    Ok(())
}

fn visit_var_decl<'a>(var_decl: &'a VarDecl<'a>, env: &mut Environment<'a>) -> miette::Result<()> {
    match &var_decl.expr {
        Some(e) => {
            let value = eval_expr(&e, env)?;
            env.define(var_decl.ident, value);
        }
        None => env.define(var_decl.ident, Atom::Nil),
    };

    Ok(())
}

#[test]
fn print() {
    let program = Parser::new(r#"print "Hello world!";"#).parse().unwrap();
    let mut stdout = Vec::new();
    run(&program, &mut stdout).unwrap();
    assert_eq!(&stdout, b"Hello world!")
}

#[test]
fn var_decl() -> miette::Result<()> {
    let program = Parser::new("var a = 3; print a;").parse()?;
    let mut stdout = Vec::new();
    run(&program, &mut stdout)?;
    assert_eq!(&stdout, b"3.0");

    Ok(())
}
