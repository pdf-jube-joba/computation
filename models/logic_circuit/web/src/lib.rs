use logic_circuit_core::machine::{
    concat_inpin, concat_otpin, Graph, InPin, LogicCircuit, LogicCircuitTrait, OtPin,
};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use utils::{alphabet::Identifier, bool::Bool, ToJsResult};
use wasm_bindgen::prelude::*;

// many global mutable turing machines
static MACHINES: LazyLock<Mutex<Vec<LogicCircuit>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    pub state: bool,
}

#[wasm_bindgen]
impl PinWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, state: bool) -> Self {
        Self { name, state }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub kind: String,
    #[wasm_bindgen(getter_with_clone)]
    pub inpins: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub otpins: Vec<PinWeb>,
    // kind が Gate のときのみ Some
    #[wasm_bindgen(getter_with_clone)]
    pub state: Option<bool>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub from: String,
    #[wasm_bindgen(getter_with_clone)]
    pub otpin: String,
    #[wasm_bindgen(getter_with_clone)]
    pub to: String,
    #[wasm_bindgen(getter_with_clone)]
    pub inpin: String,
}

#[wasm_bindgen]
pub struct GraphWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub boxes: Vec<BoxWeb>,
    #[wasm_bindgen(getter_with_clone)]
    // ignore the state
    pub inpins: Vec<PinWeb>,
    #[wasm_bindgen(getter_with_clone)]
    pub otpins: Vec<PinWeb>,
    #[wasm_bindgen(getter_with_clone)]
    pub edges: Vec<EdgeWeb>,
}

#[wasm_bindgen]
pub struct PinMapping {
    inpins_map: HashMap<InPin, (Identifier, InPin)>,
    otpins_map: HashMap<OtPin, (Identifier, OtPin)>,
}

fn lc_to_box(lc: &LogicCircuit, name: String) -> BoxWeb {
    let mut inpins = vec![];
    let mut otpins = vec![];
    for pin in lc.get_inpins() {
        inpins.push(pin.to_string());
    }
    for (pin, b) in lc.get_otputs() {
        otpins.push(PinWeb {
            name: pin.to_string(),
            state: b.into(),
        });
    }
    BoxWeb {
        kind: lc.kind().to_string(),
        name,
        inpins,
        otpins,
        state: lc.state_as_gate().map(|b| b.into()),
    }
}

fn graph_to_graphweb(g: Graph) -> GraphWeb {
    let Graph {
        verts,
        edges,
        inpins_map,
        otpins_map,
    } = g;
    let mut inpins: Vec<(Identifier, InPin, bool)> = vec![];
    let mut otpins: Vec<(Identifier, OtPin, bool)> = vec![];
    let mut boxes = vec![];
    let mut edges_web = vec![];
    for (name, lc) in &verts {
        for inpin in lc.get_inpins() {
            inpins.push((name.clone(), inpin, false));
        }
        for (otpin, b) in lc.get_otputs() {
            otpins.push((name.clone(), otpin, b == Bool::T));
        }
        boxes.push(lc_to_box(lc, name.to_string()));
    }
    for ((from, otpin), (to, inpin)) in edges {
        if let Some(pos) = inpins.iter().position(|(n, i, _)| *n == to && *i == inpin) {
            inpins.remove(pos);
        }
        if let Some(pos) = otpins
            .iter()
            .position(|(n, o, _)| *n == from && *o == otpin)
        {
            otpins.remove(pos);
        }
        edges_web.push(EdgeWeb {
            from: from.to_string(),
            otpin: otpin.to_string(),
            to: to.to_string(),
            inpin: inpin.to_string(),
        });
    }
    GraphWeb {
        boxes,
        inpins: inpins
            .into_iter()
            .map(|(name, pin, state)| PinWeb {
                name: concat_inpin(name, pin).to_string(),
                state,
            })
            .collect(),
        otpins: otpins
            .into_iter()
            .map(|(name, pin, state)| PinWeb {
                name: concat_otpin(name, pin).to_string(),
                state,
            })
            .collect(),
        edges: edges_web,
    }
}

#[wasm_bindgen]
pub fn new_logic_circuit(code: String) -> Result<usize, String> {
    let lc = logic_circuit_core::manipulation::parse_main(code.as_str()).to_js()?;
    let mut machines = MACHINES.lock().unwrap();
    machines.push(lc);
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_logic_circuit(id: usize, code: String) -> Result<(), String> {
    let lc = logic_circuit_core::manipulation::parse_main(code.as_str()).to_js()?;

    let mut machines = MACHINES.lock().unwrap();
    if id >= machines.len() {
        return Err(format!("Invalid id: {}", id));
    }
    machines[id] = lc;
    Ok(())
}

#[wasm_bindgen]
pub fn get_logic_circuit(id: usize) -> Result<GraphWeb, String> {
    let machines = MACHINES.lock().unwrap();
    if id >= machines.len() {
        return Err(format!("Invalid id: {}", id));
    }
    let lc = &machines[id];
    Ok(graph_to_graphweb(lc.as_graph_group()))
}

#[wasm_bindgen]
pub fn step_logic_circuit(id: usize, inputs: Vec<PinWeb>) -> Result<(), String> {
    let mut machines = MACHINES.lock().unwrap();
    if id >= machines.len() {
        return Err(format!("Invalid id: {}", id));
    }
    let lc = &mut machines[id];
    let inputs: Vec<(InPin, Bool)> = inputs
        .into_iter()
        .map(|pin| {
            let name: InPin = pin.name.parse()?;
            let state: Bool = if pin.state { Bool::T } else { Bool::F };
            Ok((name, state))
        })
        .collect::<Result<_, _>>()
        .to_js()?;
    lc.step(inputs);
    Ok(())
}
