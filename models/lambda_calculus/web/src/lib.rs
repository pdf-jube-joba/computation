use lambda_calculus_core::machine::{
    is_normal, left_most_marked_term, left_most_reduction, LambdaTerm, MarkedTerm, Redux,
};
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;

// global mutable lambda caluclus terms
static MACHINES: LazyLock<Mutex<Vec<LambdaTerm>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct LambdaTermWasm {
    term: LambdaTerm,
}

#[wasm_bindgen]
impl LambdaTermWasm {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.term.to_string()
    }
}

#[wasm_bindgen]
pub fn parse_lambda(term: &str) -> Result<LambdaTermWasm, String> {
    let code = lambda_calculus_core::manipulation::parse::parse_lambda_read_to_end(term)?;
    Ok(LambdaTermWasm { term: code })
}

#[wasm_bindgen]
pub struct ReduxWasm {
    term: Redux,
}

#[wasm_bindgen]
impl ReduxWasm {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.term.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn var_of_term(&self) -> String {
        self.term.var.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn body_of_term(&self) -> LambdaTermWasm {
        LambdaTermWasm {
            term: self.term.body.clone(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn arg_of_term(&self) -> LambdaTermWasm {
        LambdaTermWasm {
            term: self.term.arg.clone(),
        }
    }
}

#[wasm_bindgen]
pub struct MarkedTermWasm {
    term: MarkedTerm,
}

#[wasm_bindgen]
impl MarkedTermWasm {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.term.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        match self.term {
            MarkedTerm::Redux(_) => "redux".to_string(),
            MarkedTerm::Abstraction(_, _) => "abstraction".to_string(),
            MarkedTerm::ApplicationL(_, _) => "applicationL".to_string(),
            MarkedTerm::ApplicationR(_, _) => "applicationR".to_string(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn as_redux(&self) -> Option<ReduxWasm> {
        if let MarkedTerm::Redux(redux) = &self.term {
            Some(ReduxWasm {
                term: redux.clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn abs_var(&self) -> Option<String> {
        if let MarkedTerm::Abstraction(var, _) = &self.term {
            Some(var.to_string())
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn abs_body(&self) -> Option<MarkedTermWasm> {
        if let MarkedTerm::Abstraction(_, body) = &self.term {
            Some(MarkedTermWasm {
                term: body.as_ref().clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn app_left(&self) -> Option<MarkedTermWasm> {
        if let MarkedTerm::ApplicationL(left, _) = &self.term {
            Some(MarkedTermWasm {
                term: left.as_ref().clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn app_left_else(&self) -> Option<LambdaTermWasm> {
        if let MarkedTerm::ApplicationL(_, body) = &self.term {
            Some(LambdaTermWasm {
                term: body.as_ref().clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn app_right(&self) -> Option<MarkedTermWasm> {
        if let MarkedTerm::ApplicationR(_, right) = &self.term {
            Some(MarkedTermWasm {
                term: right.as_ref().clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn app_right_else(&self) -> Option<LambdaTermWasm> {
        if let MarkedTerm::ApplicationR(body, _) = &self.term {
            Some(LambdaTermWasm {
                term: body.as_ref().clone(),
            })
        } else {
            None
        }
    }
}

#[wasm_bindgen]
pub fn get_marked_term(term: &LambdaTermWasm) -> Option<MarkedTermWasm> {
    left_most_marked_term(term.term.clone()).map(|marked_term| MarkedTermWasm { term: marked_term })
}

fn get_machine() -> Result<std::sync::MutexGuard<'static, Vec<LambdaTerm>>, String> {
    let machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock machines".to_string())?;
    Ok(machines)
}

#[wasm_bindgen]
pub fn new_lambda_term(term: &LambdaTermWasm) -> Result<usize, String> {
    let mut machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock machines".to_string())?;
    let term = term.term.clone();
    machines.push(term);
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_lambda_term(id: usize, term: &LambdaTermWasm) -> Result<(), String> {
    let mut machines = get_machine()?;
    machines[id] = term.term.clone();
    Ok(())
}

#[wasm_bindgen]
pub fn get_lambda_term(id: usize) -> Result<LambdaTermWasm, String> {
    let machines = get_machine()?;
    if id >= machines.len() {
        return Err(format!("Machine with id {} not found", id));
    }
    let term = &machines[id];
    Ok(LambdaTermWasm { term: term.clone() })
}

#[wasm_bindgen]
pub fn step_lambda_term(id: usize) -> Result<(), String> {
    let mut machines = get_machine()?;
    if id >= machines.len() {
        return Err(format!("Machine with id {} not found", id));
    }
    let term = &mut machines[id];
    // check if term is normal
    if is_normal(term) {
        return Err("Term is already in normal form".to_string());
    }
    // left most reduction
    let next: LambdaTerm = left_most_reduction(term.clone()).unwrap();
    *term = next;
    Ok(())
}
