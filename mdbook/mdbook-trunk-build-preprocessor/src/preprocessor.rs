use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use mdbook::{BookItem, Config};
// use scraper::Html;
use anyhow::anyhow;

use super::*;

pub struct Preprocessor {}

impl Preprocessor {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct Needed {}

impl Display for Needed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for Needed {}

impl mdbook::preprocess::Preprocessor for Preprocessor {
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
                chapter.content = replace_component(
                    &chapter.content,
                    &config.component_name,
                );
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
    Ok(PreprocessorConfig {
        component_name,
    })
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
