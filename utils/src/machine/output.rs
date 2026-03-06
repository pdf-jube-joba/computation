use crate::machine::ipc::OkBody;

pub fn print_ok_body(body: OkBody) {
    match body {
        OkBody::Pong => println!("[log] pong"),
        OkBody::Dropped => println!("[log] dropped"),
        OkBody::ModelSelected { model } => println!("[log] set: {model}"),
        OkBody::Created {
            model: _,
            create,
            snapshot,
        } => {
            println!("[create] {create}");
            println!("[snapshot] {snapshot}");
        }
        OkBody::Stepped {
            model: _,
            step,
            snapshot,
        } => {
            println!("[step] {step}");
            println!("[snapshot] {snapshot}");
        }
        OkBody::Current { model: _, snapshot } => println!("[snapshot] {snapshot}"),
    }
}
