use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::{Context, Result};

use crate::machine::ipc::{OkBody, Request, Response};

pub fn send_request(socket: &str, req: &Request) -> Result<Response> {
    let mut stream = UnixStream::connect(socket).with_context(|| {
        format!("failed to connect daemon socket: {socket} (start with `machine daemon start`)")
    })?;
    let text = serde_json::to_string(req).context("failed to serialize request")?;
    stream
        .write_all(text.as_bytes())
        .context("failed to write request")?;
    stream.write_all(b"\n").context("failed to write newline")?;
    stream.flush().context("failed to flush request")?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .context("failed to read response")?;
    if read == 0 {
        anyhow::bail!("empty response from daemon");
    }
    serde_json::from_str(line.trim_end()).context("invalid response json")
}

pub fn request_ok(socket: &str, req: &Request) -> Result<OkBody> {
    match send_request(socket, req)? {
        Response::Ok { body } => Ok(body),
        Response::Error { error } => anyhow::bail!("{error}"),
    }
}
