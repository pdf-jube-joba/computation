use clap::{Arg, ArgMatches, Command};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;
use std::process;

mod preprocessor;

pub fn make_app() -> Command {
    Command::new("mdbook-trunk-build")
        .about("A mdbook preprocessor for embedding wasm component")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args
            .get_one::<String>("renderer")
            .expect("Required argument");
        // this preprocessor supports only html
        if renderer != "html" {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing() -> Result<(), Error> {
    let preprocessor = preprocessor::Nop;
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let processed_book = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
