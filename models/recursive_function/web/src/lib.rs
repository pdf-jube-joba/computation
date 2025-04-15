use recursive_function_core::machine::{Process, RecursiveFunctions};
use std::sync::{LazyLock, Mutex};
use utils::number::NumberTuple;
use wasm_bindgen::prelude::*;

// many global mutable turing machines
static MACHINES: LazyLock<Mutex<Vec<Process>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
pub struct FunctionWeb {
    function: RecursiveFunctions,
}

#[wasm_bindgen]
impl FunctionWeb {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.function.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        match self.function {
            RecursiveFunctions::ZeroConstant => "Zero".to_string(),
            RecursiveFunctions::Successor => "SUCC".to_string(),
            RecursiveFunctions::Projection { .. } => "PROJ".to_string(),
            RecursiveFunctions::Composition { .. } => "COMP".to_string(),
            RecursiveFunctions::PrimitiveRecursion { .. } => "PRIM".to_string(),
            RecursiveFunctions::MuOperator { .. } => "MUOP".to_string(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn proj_param_len(&self) -> Option<usize> {
        if let RecursiveFunctions::Projection {
            parameter_length, ..
        } = &self.function
        {
            Some(*parameter_length)
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn proj_projection_num(&self) -> Option<usize> {
        if let RecursiveFunctions::Projection { projection_num, .. } = &self.function {
            Some(*projection_num)
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn comp_param_len(&self) -> Option<usize> {
        if let RecursiveFunctions::Composition {
            parameter_length, ..
        } = &self.function
        {
            Some(*parameter_length)
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn comp_outer_func(&self) -> Option<FunctionWeb> {
        if let RecursiveFunctions::Composition { outer_func, .. } = &self.function {
            Some(FunctionWeb {
                function: outer_func.as_ref().clone(),
            })
        } else {
            None
        }
    }

    #[wasm_bindgen(getter)]
    pub fn comp_inner_funcs(&self) -> Option<Vec<FunctionWeb>> {
        if let RecursiveFunctions::Composition { inner_funcs, .. } = &self.function {
            Some(
                inner_funcs
                    .iter()
                    .map(|func| FunctionWeb {
                        function: func.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    #[wasm_bindgen(getter)]
    pub fn prim_zero_func(&self) -> Option<FunctionWeb> {
        if let RecursiveFunctions::PrimitiveRecursion { zero_func, .. } = &self.function {
            Some(FunctionWeb {
                function: zero_func.as_ref().clone(),
            })
        } else {
            None
        }
    }

    #[wasm_bindgen(getter)]
    pub fn prim_succ_func(&self) -> Option<FunctionWeb> {
        if let RecursiveFunctions::PrimitiveRecursion { succ_func, .. } = &self.function {
            Some(FunctionWeb {
                function: succ_func.as_ref().clone(),
            })
        } else {
            None
        }
    }

    #[wasm_bindgen(getter)]
    pub fn mu_func(&self) -> Option<FunctionWeb> {
        if let RecursiveFunctions::MuOperator { mu_func } = &self.function {
            Some(FunctionWeb {
                function: mu_func.as_ref().clone(),
            })
        } else {
            None
        }
    }
}

#[wasm_bindgen]
pub struct NumberTupleWeb {
    tuple: NumberTuple,
}

#[wasm_bindgen]
impl NumberTupleWeb {
    #[wasm_bindgen(getter)]
    pub fn as_vec(&self) -> Vec<usize> {
        self.tuple.clone().into()
    }
}

#[wasm_bindgen]
pub struct ProcessWeb {
    process: Process,
}

#[wasm_bindgen]
impl ProcessWeb {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.process.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        match &self.process {
            Process::Result(_) => "result".to_string(),
            Process::Comp { .. } => "comp".to_string(),
            Process::MuOpComp { .. } => "muop".to_string(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn result(&self) -> Option<usize> {
        if let Process::Result(result) = &self.process {
            Some(result.clone().into())
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn comp_func(&self) -> Option<FunctionWeb> {
        if let Process::Comp { function, .. } = &self.process {
            Some(FunctionWeb {
                function: function.clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn comp_args(&self) -> Option<Vec<ProcessWeb>> {
        if let Process::Comp { args, .. } = &self.process {
            Some(
                args.iter()
                    .map(|arg| ProcessWeb {
                        process: arg.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn muop_now_index(&self) -> Option<usize> {
        if let Process::MuOpComp { now_index, .. } = &self.process {
            Some(now_index.clone().into())
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn muop_args(&self) -> Option<Vec<usize>> {
        if let Process::MuOpComp { args, .. } = &self.process {
            Some(args.clone().into())
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn muop_func(&self) -> Option<FunctionWeb> {
        if let Process::MuOpComp { function, .. } = &self.process {
            Some(FunctionWeb {
                function: function.clone(),
            })
        } else {
            None
        }
    }
    #[wasm_bindgen(getter)]
    pub fn muop_process(&self) -> Option<ProcessWeb> {
        if let Process::MuOpComp { process, .. } = &self.process {
            Some(ProcessWeb {
                process: process.as_ref().clone(),
            })
        } else {
            None
        }
    }
}

#[wasm_bindgen]
pub fn parse_vec(tuple: &str) -> Result<NumberTupleWeb, String> {
    let tuple = tuple.trim();
    let tuple = tuple
        .parse::<NumberTuple>()
        .map_err(|_| "parse fail".to_string())?;
    Ok(NumberTupleWeb { tuple })
}

#[wasm_bindgen]
pub fn parse_code(code: &str) -> Result<FunctionWeb, String> {
    let code = code.trim();
    let function = recursive_function_core::manipulation::parse(code)?;
    Ok(FunctionWeb { function })
}

#[wasm_bindgen]
pub fn new_machine(
    function_web: &FunctionWeb,
    number_tuple: &NumberTupleWeb,
) -> Result<usize, String> {
    let machine = Process::new(function_web.function.clone(), number_tuple.tuple.clone())?;
    let mut machines = MACHINES.lock().map_err(|_| "lock fail".to_string())?;
    machines.push(machine.clone());
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_machine(
    id: usize,
    function_web: &FunctionWeb,
    number_tuple: &NumberTupleWeb,
) -> Result<(), String> {
    let machine = Process::new(function_web.function.clone(), number_tuple.tuple.clone())?;
    let mut machines = MACHINES.lock().map_err(|_| "lock fail".to_string())?;
    if id >= machines.len() {
        return Err("id out of range".to_string());
    }
    machines[id] = machine;
    Ok(())
}

#[wasm_bindgen]
pub fn step_process(id: usize) -> Result<(), String> {
    let mut machines = MACHINES.lock().map_err(|_| "lock fail".to_string())?;
    if id >= machines.len() {
        return Err("id out of range".to_string());
    }
    let machine = &mut machines[id];
    let Some(step) = machine.eval_one_step() else {
        return Err("no step".to_string());
    };
    *machine = step;
    Ok(())
}

#[wasm_bindgen]
pub fn get_process(id: usize) -> Result<ProcessWeb, String> {
    let machines = MACHINES.lock().map_err(|_| "lock fail".to_string())?;
    if id >= machines.len() {
        return Err("id out of range".to_string());
    }
    let machine = &machines[id];
    Ok(ProcessWeb {
        process: machine.clone(),
    })
}
