use logic_circuit_core::machine::{Graph, InPin, LogicCircuit, LogicCircuitTrait};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use utils::{bool::Bool, ToJsResult};
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
pub struct PinInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub pin: String,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub from: PinInfo,
    #[wasm_bindgen(getter_with_clone)]
    pub to: PinInfo,
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
    inpins_map: HashMap<String, PinInfo>,
    otpins_map: HashMap<String, PinInfo>,
}

#[wasm_bindgen]
impl GraphWeb {
    #[wasm_bindgen]
    pub fn get_inpins_map(&self, name: &str) -> Option<PinInfo> {
        self.inpins_map.get(name).cloned()
    }
    #[wasm_bindgen]
    pub fn get_otpins_map(&self, name: &str) -> Option<PinInfo> {
        self.otpins_map.get(name).cloned()
    }
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
    let mut boxes = vec![];
    let mut edges_web = vec![];
    for (name, lc) in &verts {
        boxes.push(lc_to_box(lc, name.to_string()));
    }
    for ((from, otpin), (to, inpin)) in edges {
        edges_web.push(EdgeWeb {
            from: PinInfo {
                name: from.to_string(),
                pin: otpin.to_string(),
            },
            to: PinInfo {
                name: to.to_string(),
                pin: inpin.to_string(),
            },
        });
    }
    GraphWeb {
        boxes,
        inpins: inpins_map
            .iter()
            .map(|(epin, _)| PinWeb {
                name: epin.to_string(),
                state: false,
            })
            .collect(),
        otpins: otpins_map
            .iter()
            .map(|(epin, (no, o))| {
                // get the state of the output pin from the verts
                let b: Option<bool> = verts.iter().find_map(|(name, lc)| {
                    if name == no {
                        let otputs: Vec<_> = lc.get_otputs();
                        otputs.iter().find_map(
                            |(pin, b)| {
                                if pin == o {
                                    Some((*b).into())
                                } else {
                                    None
                                }
                            },
                        )
                    } else {
                        None
                    }
                });
                PinWeb {
                    name: epin.to_string(),
                    state: b.unwrap_or_default(),
                }
            })
            .collect(),
        edges: edges_web,
        inpins_map: inpins_map
            .iter()
            .map(|(epin, (name, pin))| {
                (
                    epin.to_string(),
                    PinInfo {
                        name: name.to_string(),
                        pin: pin.to_string(),
                    },
                )
            })
            .collect(),
        otpins_map: otpins_map
            .iter()
            .map(|(epin, (name, pin))| {
                (
                    epin.to_string(),
                    PinInfo {
                        name: name.to_string(),
                        pin: pin.to_string(),
                    },
                )
            })
            .collect(),
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
