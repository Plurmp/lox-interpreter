mod lex;
mod parse;

use std::fs::read_to_string;

use camino::Utf8PathBuf;
use clap::{Parser as ClapParser, Subcommand};
use miette::IntoDiagnostic;

use lex::Lexer;
use parse::Parser;

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
        Commands::Run { filename } => {
            todo!()
        }
    }

    Ok(())
}
