use clap::{Arg, Command};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;
use std::process;

use mdbook::{BookItem, Config};
// use scraper::Html;
use anyhow::anyhow;

pub fn make_app() -> Command {
    Command::new("mdbook-trunk-build-preprocessor")
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

    let preprocessor = MdPreprocessor::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args
            .get_one::<String>("renderer")
            .expect("Required argument");
        // this preprocessor supports only html
        if preprocessor.supports_renderer(renderer) {
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

#[derive(Debug, Default)]
pub struct MdPreprocessor {}

impl MdPreprocessor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl mdbook::preprocess::Preprocessor for MdPreprocessor {
    fn name(&self) -> &str {
        "mdbook-trunk-build"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }

    fn run(
        &self,
        ctx: &PreprocessorContext,
        mut book: Book,
    ) -> Result<Book, mdbook::errors::Error> {
        log::info!("book render start");
        let config = get_config(&ctx.config)?;

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = replace_component(&chapter.content, &config.component_name);
            }
        });
        Ok(book)
    }
}

struct PreprocessorConfig {
    component_name: String,
}

// get a config
fn get_config(config: &Config) -> Result<PreprocessorConfig, anyhow::Error> {
    let table = config
        .get("preprocessor.trunk-build-preprocessor")
        .ok_or(anyhow!("not found [preprocessor.trunk-build] field"))?
        .as_table()
        .ok_or(anyhow!("not found table under [preprocessor.trunk-build]"))?;

    let component_name = table
        .get("component-name")
        .ok_or(anyhow!("failed to find key [component-name]"))?
        .as_str()
        .ok_or(anyhow!("failed to parse as str"))?
        .to_owned();

    log::info!(
        "reading enviroment succeed: \n component_name {:?}",
        component_name
    );
    Ok(PreprocessorConfig { component_name })
}

// take a entire string
fn replace_component(str: &str, component_name: &str) -> String {
    let regex = regex::Regex::new(r#"<component\s*id\s*=\s*"(?<id>\w+)">"#).unwrap();
    let res = regex.replace_all(
        str,
        // hard code where js and wasm located
        format!(r#"<script type="module">import init from '/{}/$id.js';init('/{}/${{id}}_bg.wasm');</script><div id="$id"></div>"#, component_name, component_name),
    );
    res.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_test() {
        let str = "rec <component id=\"hello\"> rec".to_string();
        // assert!(re.captures_iter(&str).into_iter().next().is_some());
        let res = replace_component(&str, "component");
        eprintln!("{}", res);
    }
}
