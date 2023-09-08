use std::{
    fmt::Display,
    io::Write,
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
        let config = get_config(&ctx.config)?;

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = replace_component(
                    &chapter.content,
                );
            }
        });

        handle_trunk_build(config)?;
        log::info!("trunk build ended");
        Ok(book)
    }
}

struct PreprocessorConfig {
    book_src_dir: PathBuf,
    trunk_dist_dir: PathBuf,
    book_out_dir: PathBuf,
    component_dir: PathBuf,
    temp_out_dir: PathBuf,
    // temp_rel_to_book: PathBuf,
}

// get a config
fn get_config(config: &Config) -> Result<PreprocessorConfig, anyhow::Error> {
    let table = config
        .get("preprocessor.trunk-build")
        .ok_or(anyhow!("not found [preprocessor.trunk-build] field"))?
        .as_table()
        .ok_or(anyhow!("not found table under [preprocessor.trunk-build]"))?;
    let book_src_dir: PathBuf = {
        let cur = std::env::current_dir()?;
        cur.canonicalize()?
    };

    let relpath_to_abspath_from_table = |str: &str| -> Result<PathBuf, anyhow::Error> {
        let relative_dir = table
            .get(str)
            .ok_or(anyhow!("failed to get key [trunk-out-dir] in table"))?
            .as_str()
            .unwrap(); // path is written in toml file so it must succeed
        let mut path_buf = PathBuf::from(&book_src_dir);
        path_buf.extend(&PathBuf::from(relative_dir));
        path_buf.canonicalize().map_err(|err| err.into())
    };

    let trunk_dist_dir = relpath_to_abspath_from_table("trunk-out-dir")?;
    let book_out_dir = relpath_to_abspath_from_table("book-out-dir")?;
    let component_dir = relpath_to_abspath_from_table("component-dir")?;
    let temp_out_dir = relpath_to_abspath_from_table("temp-out-dir")?;
    // let temp_rel_to_book = pathdiff::diff_paths(book_out_dir.clone(), temp_out_dir.clone())
        // .ok_or(anyhow!("fail on getting relative path"))?;

    log::info!(
        "reading enviroment succeed: \n book_src_directory {:?} \n trunk_dist_dir {:?} \n book_out_dir {:?} \n component_dir {:?} \n temp_out_dir {:?}",
        book_src_dir,
        trunk_dist_dir,
        book_out_dir,
        component_dir,
        temp_out_dir,
        // temp_rel_to_book,
    );
    Ok(PreprocessorConfig {
        book_src_dir,
        trunk_dist_dir,
        book_out_dir,
        component_dir,
        temp_out_dir,
        // temp_rel_to_book,
    })
}

// build anything in component/*/src/bin/*.rs
fn handle_trunk_build(config: PreprocessorConfig) -> Result<(), anyhow::Error> {
    let PreprocessorConfig {
        book_src_dir,
        trunk_dist_dir,
        book_out_dir,
        component_dir,
        temp_out_dir,
        // temp_rel_to_book,
    } = config;

    for entry in std::fs::read_dir(component_dir)? {
        let target_dir = entry?.path();
        handle_trunk_build_mv(target_dir, &trunk_dist_dir, &temp_out_dir)?;
    }
    Ok(())
}

fn handle_trunk_build_mv(
    target_dir: PathBuf,
    trunk_dist_dir: &Path,
    temp_out_dir: &Path,
) -> Result<(), anyhow::Error> {
    log::info!("trunk build directory: {:?}", target_dir);

    std::env::set_current_dir(target_dir.clone())?;

    let glob_target_file: PathBuf = {
        let mut dir = target_dir.clone();
        dir.extend(vec!["src", "bin", "*.rs"]);
        dir
    };

    let mut build_command = Command::new("trunk");
    build_command
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for entry in glob::glob(&format!("{}", glob_target_file.as_path().display()))? {
        let file_name = entry?;
        // if let Ok(file_name) = entry {
        log::info!("target_file: {:?}", file_name);

        let file_stem = file_name
            .file_stem()
            .ok_or(anyhow!("failed to get file_stem :{file_name:?}"))?
            .to_str()
            .ok_or(anyhow!("failed to convert OsStr to str"))?;

        log::info!("write index_html");
        let mut index_html_file = std::fs::File::create("./index.html")?;
        index_html_file.write_all(trunk_build_html(file_stem).as_bytes())?;

        log::info!("invoke trunk build");
        if build_command.spawn()?.wait()?.success() {
            log::info!("build succeed");
        } else {
            log::info!("build failed");
            return Err(anyhow!("build failed"));
        }

        let target_files = {
            let mut path_buf: PathBuf = trunk_dist_dir.to_path_buf();
            path_buf.push(format!("{file_stem}*"));
            path_buf
        };

        log::info!("move generated file");
        for entry in glob::glob(&format!("{}", target_files.as_path().display()))? {
            let file_name = entry?;
            log::info!("cp file:{:?}", file_name);
            let mut mv_command = Command::new("cp");
            mv_command
                .args([
                    format!("{}", file_name.as_path().display()),
                    format!("{}/", temp_out_dir.to_path_buf().display()),
                ])
                .spawn()?
                .wait()?;
        }
    }
    Ok(())
}

// trunk need a index.html so generate
fn trunk_build_html(name: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html><html>
    <head></head><body>
    <link data-trunk rel="rust" data-bin="{name}" data-type="main"/>
    </body></html>
    "#
    )
}

// take a entire string
fn replace_component(str: &str) -> String {
    let regex = regex::Regex::new(r#"<component\s*id\s*=\s*"(?<id>\w+)">"#).unwrap();
    let res = regex.replace_all(
        str,
        // hard code where js and wasm located
        r#"<script type="module">import init from '/js/$id.js';init('/js/${id}_bg.wasm');</script>
        <div id="$id"></div>"#,
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
        let res = replace_component(&str);
        eprintln!("{}", res);
    }
}
