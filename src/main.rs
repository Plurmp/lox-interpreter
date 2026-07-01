mod eval;
mod lex;
mod parse;
mod run;

use std::fs::read_to_string;

use camino::Utf8PathBuf;
use clap::{Parser as ClapParser, Subcommand};
use miette::IntoDiagnostic;

use eval::eval_expr;
use lex::Lexer;
use parse::Parser;

use crate::run::Environment;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "kebab-case")]
enum Commands {
    Tokenize {
        filename: Utf8PathBuf,
    },
    Parse {
        filename: Utf8PathBuf,
        #[arg(short, long)]
        expression: bool,
    },
    Evaluate {
        filename: Utf8PathBuf,
    },
    Run {
        filename: Utf8PathBuf,
    },
}

fn main() -> miette::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Tokenize { filename } => {
            let file_contents = read_to_string(filename).into_diagnostic()?;

            for token in Lexer::new(&file_contents) {
                let token = match token {
                    Ok(t) => t.to_string(),
                    Err(e) => e.to_string(),
                };
                println!("{token}")
            }
            println!("EOF  null")
        }
        Commands::Parse {
            filename,
            expression,
        } => {
            let file_contents = read_to_string(filename).into_diagnostic()?;

            let mut parser = Parser::new(&file_contents);
            if expression {
                let parse_tree = parser.expr()?;
                println!("{parse_tree}");
            } else {
                todo!()
            };
        }
        Commands::Evaluate { filename } => {
            let file_contents = read_to_string(filename).into_diagnostic()?;

            let mut env = Environment::new();
            let mut parser = Parser::new(&file_contents);
            let expr = parser.expr()?;
            println!("{}", eval_expr(&expr, &mut env)?);
        }
        Commands::Run { filename } => {
            let file_contents = read_to_string(filename).into_diagnostic()?;

            let program = Parser::new(&file_contents).parse()?;
            run::run(&program, &mut std::io::stdout())?;
        }
    }

    Ok(())
}
