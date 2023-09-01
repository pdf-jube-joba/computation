use std::{path::PathBuf, process::{Command, Stdio}, fmt::Display, io::Write};

use mdbook::{book::Chapter, BookItem, Config};
use scraper::Html;

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

impl std::error::Error for Needed {
}

impl mdbook::preprocess::Preprocessor for Preprocessor {
    fn name(&self) -> &str {
        "mdbook-trunk-build"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, mdbook::errors::Error> {
        let mut names = vec![];
        book.for_each_mut(|item|{
            if let BookItem::Chapter(chapter) = item {
                let (modified, name_in_chapter) = change_and_collect(&chapter.content);
                chapter.content = modified;
                names.extend(name_in_chapter)
            }
        });

        handle_trunk_build(names, &ctx.config).map_err(|_| mdbook::errors::Error::new( Needed {}))?;
        unimplemented!()
    }
}

// help me
fn handle_trunk_build(names: Vec<String>, config: &Config) -> Result<(), ()> {
    let table = config.get("preprocessor.trunk-build").ok_or(())?
        .as_table().ok_or(())?;
    eprintln!("{table:?}");
    let root_of_entire_directory: PathBuf = {
        let cur = std::env::current_dir().unwrap();
        // cur.parent().unwrap().to_path_buf()
        cur.canonicalize().map_err(|_|())?
    };
    eprintln!("{root_of_entire_directory:?} \n");

    let trunk_out_dir = {
        let rel_dir = table.get("trunk-out-dir").ok_or(())?.as_str().ok_or(())?;
        let mut path_buf = PathBuf::new();
        path_buf.extend(&root_of_entire_directory);
        path_buf.extend(&PathBuf::from(rel_dir));
        path_buf.canonicalize().map_err(|_|())?
    };
    let book_out_dir = {
        let rel_dir = table.get("book-out-dir").ok_or(())?.as_str().ok_or(())?;
        let mut path_buf = PathBuf::new();
        path_buf.extend(&root_of_entire_directory);
        path_buf.extend(&PathBuf::from(rel_dir));
        path_buf.canonicalize().map_err(|_|())?
    };
    let component_dir = {
        let rel_dir = table.get("component-dir").ok_or(())?.as_str().ok_or(())?;
        let mut path_buf = PathBuf::new();
        path_buf.extend(&root_of_entire_directory);
        path_buf.extend(&PathBuf::from(rel_dir));
        path_buf.canonicalize().map_err(|_|())?
    };
    eprintln!("component-dir:{component_dir:?}\ntrunk-out-dir:{trunk_out_dir:?}\nbook-out-dir:{book_out_dir:?}\n");

    std::env::set_current_dir(component_dir.clone()).map_err(|_|())?;

    for entry in std::fs::read_dir(component_dir).map_err(|_| ())? {
        let dir = entry.map_err(|_| ())?;
        std::env::set_current_dir(dir.path()).map_err(|_| ())?;

        let mut build_command = Command::new("trunk");
        build_command.arg("build");

        for entry in glob::glob("./src/bin/*.rs").map_err(|_|())? {
            if let Ok(file_name) = entry {
                let file_stem = file_name.file_stem().ok_or(())?.to_str().unwrap();
                let mut index_html = std::fs::File::create("./index.html").map_err(|_|())?;
                index_html.write_all(trunk_build_html(file_stem).as_bytes()).map_err(|_|())?;
                eprintln!("now dir file {:?} {:?} \n", std::env::current_dir().unwrap(), file_stem);
                let _ = build_command.spawn().map_err(|_|())?.wait().map_err(|_|())?;
                let target_files = {
                    let mut path_buf: PathBuf = PathBuf::from(trunk_out_dir.clone());
                    path_buf.push(format!("{file_stem}*"));
                    path_buf
                };
                for entry in glob::glob(&format!("{}", target_files.as_path().display())).map_err(|_|())? {
                    if let Ok(file_name) = entry {
                        let mut mv_command = Command::new("mv");
                        mv_command
                        .args([
                            format!("{}", file_name.as_path().display()),
                            format!("{}/", book_out_dir.as_path().display()),
                        ])
                        .spawn().map_err(|_|())?.wait().map_err(|_|())?;                
                    }
                }
            }
        }
    }
    Ok(())
}

// trunk need a index.html so generate
fn trunk_build_html(name: &str) -> String {
    format!(r#"
    <!DOCTYPE html><html>
    <head></head><body>
    <link data-trunk rel="rust" data-bin="{name}" data-type="main"/>
    </body></html>
    "#)
}

// take a entire string
fn change_and_collect(str: &str) -> (String, Vec<String>) {
    // TODO inefficients implementations
    let regex = regex::Regex::new(r#"<component\s*id\s*=\s*"(\w+)""#).unwrap();
    let mut v = vec![];
    for (_, [id]) in regex.captures_iter(str).map(|cap| cap.extract()) {
        v.push(id.to_string());
    }
    let res = regex.replace_all(str, {
        format!(r#"<script type="module">import init from '/$0.js';init('/$0_bg.wasm');</script>\n<div id="$0"></div>"#)
    });
    (res.into(), v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_test() {
        let mut str="rec <component id=\"hello\"> rec".to_string();
        // assert!(re.captures_iter(&str).into_iter().next().is_some());
        let res = change_and_collect(&mut str);
        eprintln!("{}", res.0);
        let exp = vec!["hello".to_string()];
        assert_eq!(res.1, exp);
    }
}
