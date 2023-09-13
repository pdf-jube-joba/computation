use anyhow::anyhow;
use mdbook::{renderer::RenderContext, Config};
use std::env::set_current_dir;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::{
    io,
    path::{Path, PathBuf},
};

fn main() {
    env_logger::builder()
        .target(env_logger::Target::Stderr)
        .init();
    log::info!("start trunk build renderer");
    if let Err(err) = run() {
        log::error!("{err}");
        std::process::exit(1);
    }
    log::info!("end trunk build renderer");
}

fn run() -> Result<(), anyhow::Error> {
    let mut stdin = io::stdin();
    let ctx = RenderContext::from_json(&mut stdin).unwrap();
    let config = handle_config(&ctx.config)?;
    handle_trunk(config)
}

struct RendererConfig {
    component_dir: PathBuf,
    trunk_out_dir: PathBuf,
    component_out_dir: PathBuf,
}

fn handle_config(config: &Config) -> Result<RendererConfig, anyhow::Error> {
    log::info!("start reading config");

    // get a table from book.toml table
    let table = config
        .get("output.trunk-build-renderer")
        .ok_or(anyhow!("not found [preprocessor.trunk-build] field"))?
        .as_table()
        .ok_or(anyhow!("not found table under [preprocessor.trunk-build]"))?;

    // get a book_src_dir
    // remark: mdbook invoke a renderer in a out_dir
    let book_src_dir: PathBuf = {
        let cur = std::env::current_dir()?;
        cur.canonicalize()?
            .parent()
            .and_then(|dir| dir.parent())
            .and_then(|dir| dir.parent())
            .ok_or(anyhow!("parent dir not found"))?
            .into()
    };

    // set current directory to book_src_dir
    log::debug!("book_src_dir {:?}", book_src_dir);
    set_current_dir(book_src_dir.clone())
        .map_err(|err| anyhow!("fail to set current directory to book_src_dir {err}"))?;

    let relpath_to_abspath_from_table = |str: &str| -> Result<PathBuf, anyhow::Error> {
        // get a relative path from table
        let relative_dir = table
            .get(str)
            .ok_or(anyhow!("failed to get key ({str}) in table"))?
            .as_str()
            .ok_or(anyhow!("failed to read value as str"))?;

        log::debug!("set cur dir");
        // make sure that the target directory is exists
        let _ = fs::create_dir(relative_dir);

        // return absolute path to target directory
        let mut path_buf = PathBuf::from(&book_src_dir);
        path_buf.extend(&PathBuf::from(relative_dir));
        path_buf.canonicalize().map_err(|err| err.into())
    };

    log::debug!("get directory");

    let component_dir = relpath_to_abspath_from_table("component-dir")?;
    let trunk_out_dir = relpath_to_abspath_from_table("trunk-out-dir")?;
    let component_out_dir = relpath_to_abspath_from_table("component-out-dir")?;

    log::info!(
        "reading enviroment succeed: \n component_dir {:?} \n component_out_dir {:?} \n trunk_out_dir {:?} \n ",
        component_dir,
        component_out_dir,
        trunk_out_dir,
    );
    Ok(RendererConfig {
        component_dir,
        trunk_out_dir,
        component_out_dir,
    })
}

// build anything in component/*/src/bin/*.rs
fn handle_trunk(config: RendererConfig) -> Result<(), anyhow::Error> {
    let RendererConfig {
        trunk_out_dir,
        component_out_dir,
        component_dir,
    } = config;

    for entry in std::fs::read_dir(component_dir)? {
        let target_dir = entry?.path();
        std::env::set_current_dir(target_dir.clone())?;
        log::debug!("target directory {target_dir:?}");
        let glob_target_file: PathBuf = {
            let mut dir = target_dir.clone();
            dir.extend(vec!["src", "bin", "*.rs"]);
            dir
        };
        log::debug!("get glob {target_dir:?}");
        for entry in glob::glob(&format!("{}", glob_target_file.as_path().display()))
            .map_err(|err| anyhow!("glo fail {err}"))?
        {
            let file_name = entry?;
            // if let Ok(file_name) = entry {
            log::debug!("target_file: {:?}", file_name);

            let target_file_stem = file_name
                .file_stem()
                .ok_or(anyhow!("failed to get file_stem :{file_name:?}"))?
                .to_str()
                .ok_or(anyhow!("failed to convert OsStr to str"))?;
            log::debug!("get filestem :{target_file_stem}");
            handle_one_file(
                &target_dir,
                target_file_stem,
                &trunk_out_dir,
                &component_out_dir,
            )?;
        }
    }
    Ok(())
}

fn handle_one_file(
    target_dir: &PathBuf,
    target_file_stem: &str,
    trunk_out_dir: &Path,
    component_out_dir: &Path,
) -> Result<(), anyhow::Error> {
    log::info!("start file {target_dir:?} {target_file_stem}");
    if !target_dir.is_absolute() {
        return Err(anyhow!("not a absolute path {target_dir:?}"));
    }
    std::env::set_current_dir(target_dir)?;

    log::debug!("write index_html");
    let mut index_html_file = std::fs::File::create("./index.html")?;
    index_html_file.write_all(index_html_for_trunk_build(target_file_stem).as_bytes())?;
    log::debug!("writed index_html");

    log::info!("invoke trunk build");
    let mut build_command = Command::new("trunk");
    let result = build_command.arg("build").spawn()?.wait()?;
    if result.success() {
        log::info!("build succeed");
    } else {
        log::info!("build failed");
        return Err(anyhow!("build failed"));
    }

    log::info!("move generated file");
    let target_files_glob = {
        let mut path_buf: PathBuf = trunk_out_dir.to_path_buf();
        path_buf.push(format!("{target_file_stem}*"));
        path_buf
    };
    for entry in glob::glob(&format!("{}", target_files_glob.as_path().display()))? {
        let file_name = entry?;
        log::info!("cp file:{:?}", file_name);
        let mut mv_command = Command::new("cp");
        mv_command
            .args([
                format!("{}", file_name.as_path().display()),
                format!("{}/", component_out_dir.to_path_buf().display()),
            ])
            .spawn()?
            .wait()?;
    }
    Ok(())
}

// trunk need a index.html so generate
fn index_html_for_trunk_build(name: &str) -> String {
    format!(
        r#"
           <!DOCTYPE html><html>
           <head></head><body>
           <link data-trunk rel="rust" data-bin="{name}" data-type="main"/>
           </body></html>
        "#
    )
}
