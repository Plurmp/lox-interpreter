use miette::IntoDiagnostic;

use crate::{
    eval::eval_expr,
    parse::{Program, Statement},
    Parser,
};

pub fn run<W: std::io::Write>(program: &Program, stdout: &mut W) -> miette::Result<()> {
    for statement in &program.statements {
        match statement {
            Statement::PrintStatement(expr) => {
                write!(stdout, "{}", eval_expr(&expr)?).into_diagnostic()?
            }
            _ => todo!(),
        }
    }

    Ok(())
}

#[test]
fn print() {
    let program = Parser::new(r#"print "Hello world!";"#).parse().unwrap();
    let mut stdout = Vec::new();
    run(&program, &mut stdout).unwrap();
    assert_eq!(&stdout, b"Hello world!")
}
