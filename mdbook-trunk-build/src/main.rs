use clap::{Arg, Command};
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
    env_logger::builder()
        .target(env_logger::Target::Stderr)
        .init();

    let matches = make_app().get_matches();

    let preprocessor = preprocessor::Preprocessor::new(

    );

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args
            .get_one::<String>("renderer")
            .expect("Required argument");
        // this preprocessor supports only html
        if preprocessor.supports_renderer(&renderer) {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(preprocessor: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let processed_book = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
