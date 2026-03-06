use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use utils::machine_ipc::{Request, Response};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

mod web_model {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "web-model",
    });
}

#[derive(Parser, Debug)]
#[command(about = "Daemon for one-shot machine CLI over Unix socket")]
struct Args {
    #[arg(long, default_value = "/tmp/computation-machine.sock")]
    socket: String,
}

struct ModelSession {
    name: String,
    store: Store<()>,
    instance: web_model::WebModel,
}

struct DaemonState {
    engine: Engine,
    current: Option<ModelSession>,
}

fn resolve_component_path(name: &str) -> PathBuf {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("utils has workspace parent");
    workspace
        .join("wasm_bundle")
        .join(format!("{name}.component.wasm"))
}

fn new_engine() -> Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    Engine::new(&config).context("failed to initialize wasmtime engine")
}

fn expect_ok<T>(label: &str, value: std::result::Result<T, String>) -> Result<T> {
    value.map_err(|e| anyhow::anyhow!("{label}: {e}"))
}

fn parse_request(stream: &UnixStream) -> Result<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .context("failed to read request line")?;
    if read == 0 {
        anyhow::bail!("empty request");
    }
    serde_json::from_str(line.trim_end()).context("invalid request json")
}

fn write_response(mut stream: UnixStream, resp: &Response) -> Result<()> {
    let text = serde_json::to_string(resp).context("failed to serialize response")?;
    stream
        .write_all(text.as_bytes())
        .context("failed to write response")?;
    stream.write_all(b"\n").context("failed to write newline")?;
    stream.flush().context("failed to flush response")
}

fn with_session_mut(state: &mut DaemonState) -> Result<&mut ModelSession> {
    state
        .current
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("no model selected; run `machine model <name>` first"))
}

fn handle_request(state: &mut DaemonState, req: Request) -> Response {
    match dispatch_request(state, req) {
        Ok(resp) => resp,
        Err(err) => Response::err(err.to_string()),
    }
}

fn dispatch_request(state: &mut DaemonState, req: Request) -> Result<Response> {
    match req {
        Request::Ping => Ok(Response::ok_message("pong")),
        Request::Drop => {
            state.current = None;
            Ok(Response::ok_message("dropped"))
        }
        Request::Model { name } => {
            let component_path = resolve_component_path(&name);
            if !component_path.exists() {
                anyhow::bail!("component not found: {}", component_path.display());
            }

            let component = Component::from_file(&state.engine, &component_path)
                .with_context(|| format!("failed to load component: {}", component_path.display()))?;
            let linker = Linker::new(&state.engine);
            let mut store = Store::new(&state.engine, ());
            let instance = web_model::WebModel::instantiate(&mut store, &component, &linker)
                .context("instantiate as web-model failed")?;

            state.current = Some(ModelSession {
                name: name.clone(),
                store,
                instance,
            });

            Ok(Response::Ok {
                message: Some("set".to_string()),
                model: Some(name),
                create: None,
                step: None,
                snapshot: None,
            })
        }
        Request::Create { code, ainput } => {
            let session = with_session_mut(state)?;
            let create = expect_ok(
                "web-model create",
                session
                    .instance
                    .call_create(&mut session.store, &code, &ainput)
                    .context("web-model create failed")?,
            )?;
            let snapshot = expect_ok(
                "web-model current-machine",
                session
                    .instance
                    .call_current_machine(&mut session.store)
                    .context("web-model current-machine failed")?,
            )?;
            Ok(Response::Ok {
                message: None,
                model: Some(session.name.clone()),
                create: Some(create),
                step: None,
                snapshot: Some(snapshot),
            })
        }
        Request::Step { rinput } => {
            let session = with_session_mut(state)?;
            let step = expect_ok(
                "web-model step-machine",
                session
                    .instance
                    .call_step_machine(&mut session.store, &rinput)
                    .context("web-model step-machine failed")?,
            )?;
            let snapshot = expect_ok(
                "web-model current-machine",
                session
                    .instance
                    .call_current_machine(&mut session.store)
                    .context("web-model current-machine failed")?,
            )?;
            Ok(Response::Ok {
                message: None,
                model: Some(session.name.clone()),
                create: None,
                step: Some(step),
                snapshot: Some(snapshot),
            })
        }
        Request::Current => {
            let session = with_session_mut(state)?;
            let snapshot = expect_ok(
                "web-model current-machine",
                session
                    .instance
                    .call_current_machine(&mut session.store)
                    .context("web-model current-machine failed")?,
            )?;
            Ok(Response::Ok {
                message: None,
                model: Some(session.name.clone()),
                create: None,
                step: None,
                snapshot: Some(snapshot),
            })
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let socket_path = Path::new(&args.socket);
    if socket_path.exists() {
        fs::remove_file(socket_path)
            .with_context(|| format!("failed to remove stale socket: {}", socket_path.display()))?;
    }

    let listener = UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind socket: {}", socket_path.display()))?;
    eprintln!("[machine-daemon] listening: {}", socket_path.display());

    let engine = new_engine()?;
    let mut state = DaemonState {
        engine,
        current: None,
    };

    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let resp = match parse_request(&stream) {
                    Ok(req) => handle_request(&mut state, req),
                    Err(err) => Response::err(err.to_string()),
                };
                if let Err(err) = write_response(stream, &resp) {
                    eprintln!("[machine-daemon] write error: {err:#}");
                }
            }
            Err(err) => {
                eprintln!("[machine-daemon] accept error: {err}");
            }
        }
    }

    Ok(())
}
