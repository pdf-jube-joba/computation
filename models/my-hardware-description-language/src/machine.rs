use std::collections::{HashMap, HashSet};

use anyhow::{bail, Error};
use either::Either::{self, Left, Right};
use utils::{bool::Bool, number::Number};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    Bit,
    Array(Number, Box<ValueType>),
    Strct(Vec<(String, ValueType)>),
    Enume(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Bit(Bool),
    Array(Vec<Value>),
    Strct(Vec<(String, Value)>),
    Enume(String),
}

impl Value {
    pub fn get_field_of_value(&self, str: &str) -> Option<&Value> {
        let Value::Strct(v) = self else {
            return None;
        };
        v.iter()
            .find_map(|(s, v)| if *s == str { Some(v) } else { None })
    }
}

fn typable_value(value: &Value, type_expect: &ValueType) -> Result<(), Error> {
    match (value, type_expect) {
        (Value::Bit(_), ValueType::Bit) => Ok(()),
        (Value::Array(v), ValueType::Array(len, val_type)) => {
            let v_len: Number = v.len().into();
            if v_len != *len {
                bail!("mismatch length of array")
            }
            for vi in v.iter() {
                typable_value(vi, val_type)?;
            }
            Ok(())
        }
        (Value::Strct(val), ValueType::Strct(val_type)) => {
            let val_keys: HashSet<_> = val.iter().map(|(s, _)| s.clone()).collect();
            let val_type_keys: HashSet<_> = val_type.iter().map(|(s, _)| s.clone()).collect();
            if val_keys != val_type_keys {
                bail!("mismatch field name");
            }
            for k in val_keys {
                let v = val
                    .iter()
                    .find_map(|(s, v)| if *s == k { Some(v.clone()) } else { None })
                    .unwrap();
                let t = val_type
                    .iter()
                    .find_map(|(s, v)| if *s == k { Some(v.to_owned()) } else { None })
                    .unwrap();
                typable_value(&v, &t)?;
            }
            Ok(())
        }
        (Value::Enume(val), ValueType::Enume(vals)) => {
            if vals.contains(val) {
                Ok(())
            } else {
                bail!("enum {val} is not contained in {vals:?}")
            }
        }
        _ => {
            bail!("type mismatch")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CombExpVal {
    Val(Value),
    Var(String),
    And(Box<CombExpVal>, Box<CombExpVal>),
    Or(Box<CombExpVal>, Box<CombExpVal>),
    Not(Box<CombExpVal>),
    IfCond(Box<CombExpVal>, Box<CombExpVal>, Box<CombExpVal>),
    ArrayAcc(Box<CombExpVal>, Number),
    ArrayConst(Vec<CombExpVal>),
    FieldAcc(Box<CombExpVal>, String),
    StrctConst(Vec<(String, CombExpVal)>),
    Switch(Box<CombExpVal>, Vec<(String, CombExpVal)>),
    SeqAndFinal(Vec<(String, CombExpVal)>, Box<CombExpVal>),
    CombModule(String, Box<CombExpVal>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CombModule {
    name: String,
    input_type: ValueType,
    otput_type: ValueType,
    comb_func: CombExpVal,
}

fn eval_to_val(
    mod_env: &Vec<CombModule>,
    var_env: &Vec<(String, Value)>,
    exp: &CombExpVal,
) -> Result<Value, Error> {
    match exp {
        CombExpVal::Val(v) => Ok(v.clone()),
        CombExpVal::Var(s) => {
            match var_env
                .iter()
                .find_map(|(s0, v)| if s == s0 { Some(v.clone()) } else { None })
            {
                Some(v) => Ok(v),
                None => bail!("not found var {s}"),
            }
        }
        CombExpVal::And(e1, e2) => {
            let v1 = eval_to_val(mod_env, var_env, e1.as_ref())?;
            let v2 = eval_to_val(mod_env, var_env, e2.as_ref())?;
            match (v1, v2) {
                (Value::Bit(b1), Value::Bit(b2)) => Ok(Value::Bit(b1.and(b2))),
                _ => bail!("expression is not boolean"),
            }
        }
        CombExpVal::Or(e1, e2) => {
            let v1 = eval_to_val(mod_env, var_env, e1)?;
            let v2 = eval_to_val(mod_env, var_env, e2)?;
            match (v1, v2) {
                (Value::Bit(b1), Value::Bit(b2)) => Ok(Value::Bit(b1.or(b2))),
                _ => bail!("expression is not boolean"),
            }
        }
        CombExpVal::Not(e) => {
            let v = eval_to_val(mod_env, var_env, e)?;
            match v {
                Value::Bit(b) => Ok(Value::Bit(!b)),
                _ => bail!("expression is not boolean"),
            }
        }
        CombExpVal::IfCond(e0, e1, e2) => {
            let v0 = eval_to_val(mod_env, var_env, e0)?;
            let v1 = eval_to_val(mod_env, var_env, e1)?;
            let v2 = eval_to_val(mod_env, var_env, e2)?;
            match v0 {
                Value::Bit(Bool::T) => Ok(v1),
                Value::Bit(Bool::F) => Ok(v2),
                _ => bail!("expression is not boolean"),
            }
        }
        CombExpVal::ArrayAcc(e, n) => {
            let ln: usize = n.clone().into();
            let v = eval_to_val(mod_env, var_env, e)?;
            match v {
                Value::Array(v) => {
                    if v.len() <= ln {
                        bail!("access out of index")
                    } else {
                        Ok(v[ln].clone())
                    }
                }
                _ => bail!("expression is not array"),
            }
        }
        CombExpVal::ArrayConst(e) => {
            let v: Vec<_> = e
                .iter()
                .map(|e| eval_to_val(mod_env, var_env, e))
                .collect::<Result<_, _>>()?;
            Ok(Value::Array(v))
        }
        CombExpVal::FieldAcc(e, s) => {
            let v = eval_to_val(mod_env, var_env, e)?;
            match v {
                Value::Strct(v) => {
                    for (s0, v) in v {
                        if *s == s0 {
                            return Ok(v.clone());
                        }
                    }
                    bail!("not found field {s}")
                }
                _ => bail!("expression is not struct"),
            }
        }
        CombExpVal::StrctConst(e) => {
            let v: Vec<_> = e
                .iter()
                .map(|(s, e)| -> Result<(String, Value), Error> {
                    let v = eval_to_val(mod_env, var_env, e)?;
                    Ok((s.clone(), v))
                })
                .collect::<Result<_, _>>()?;
            Ok(Value::Strct(v))
        }
        CombExpVal::Switch(e, es) => {
            let v = eval_to_val(mod_env, var_env, e)?;
            let es: Vec<(String, Value)> = es
                .iter()
                .map(|(s, e)| -> Result<(String, Value), Error> {
                    let v = eval_to_val(mod_env, var_env, e)?;
                    Ok((s.clone(), v))
                })
                .collect::<Result<_, _>>()?;
            match v {
                Value::Enume(s) => {
                    for (s0, v) in es {
                        if s == s0 {
                            return Ok(v);
                        }
                    }
                    bail!("switch miss")
                }
                _ => bail!("expression is not enume"),
            }
        }
        CombExpVal::SeqAndFinal(seq, final_val) => {
            let mut vars = var_env.clone();
            for (s, e) in seq {
                let v = eval_to_val(mod_env, &vars, e)?;
                vars.push((s.clone(), v));
            }
            eval_to_val(mod_env, &vars, final_val)
        }
        CombExpVal::CombModule(s, e) => {
            let Some(comb_mod) = mod_env.iter().find(|comb_mod| &comb_mod.name == s) else {
                bail!("module name {s} is not found")
            };
            let v = eval_to_val(mod_env, var_env, e)?;
            let v = eval_to_val(mod_env, &vec![("IN".into(), v)], &comb_mod.comb_func)?;
            Ok(v)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeEnv {
    code: CombExpVal,
    mod_env: Vec<CombModule>,
}

const FIELD_IN: &str = "IN";
const FIELD_OUT: &str = "OUT";
const FIELD_STATE: &str = "STATE";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleModule {
    state_type_and_init: (Value, ValueType),
    input_type: ValueType,
    otput_type: ValueType,
    trans_func: CombExpVal,
    otput_func: CombExpVal,
}

impl SimpleModuleState {
    fn new() -> Self {
        todo!()
    }
    fn build(&self) -> SimpleModuleState {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleModuleState {
    state: Value,
    trans_func: CodeEnv,
    otput_func: CodeEnv,
}

impl SimpleModuleState {
    fn now_state(&self) -> Value {
        self.state.clone()
    }
    fn get_otput(&self) -> Result<Value, Error> {
        eval_to_val(
            &self.otput_func.mod_env,
            &vec![(FIELD_STATE.to_string(), self.state.clone())],
            &self.otput_func.code,
        )
    }
    fn clock(&mut self, input: Value) -> Result<(), Error> {
        let state = eval_to_val(
            &self.trans_func.mod_env,
            &vec![
                (FIELD_IN.to_string(), input),
                (FIELD_STATE.to_string(), self.state.clone()),
            ],
            &self.trans_func.code,
        )?;
        self.state = state;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneralModule {
    Simple(Box<SimpleModule>),
    Graph(Box<GraphModule>),
    Iter(Box<IterModule>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneralModuleState {
    Simple(Box<SimpleModuleState>),
    Graph(Box<GraphModuleState>),
    Iter(Box<IterModuleState>),
}

impl GeneralModuleState {
    pub fn now_state(&self) -> Value {
        todo!()
    }
    pub fn get_otput(&self) -> Result<Value, Error> {
        todo!()
    }
    pub fn clock(&mut self, input: Value) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GraphModule {
    state_name_machines: Vec<(String, GeneralModule)>,
    input_type: ValueType,
    otput_type: ValueType,
    trans_func: CombExpVal,
    otput_func: CombExpVal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GraphModuleState {
    states_machines: Vec<(String, GeneralModuleState)>,
    trans_func: CodeEnv,
    otput_func: CodeEnv,
}

impl GraphModuleState {
    fn now_state(&self) -> Value {
        let vars = self
            .states_machines
            .iter()
            .map(|(s, sm)| (s.to_string(), sm.now_state()))
            .collect();
        Value::Strct(vars)
    }
    fn get_otput(&self) -> Result<Value, Error> {
        let vars = self
            .states_machines
            .iter()
            .map(|(s, sm)| -> Result<(String, Value), Error> {
                Ok((s.to_string(), sm.get_otput()?))
            })
            .collect::<Result<_, _>>()?;
        eval_to_val(&self.trans_func.mod_env, &vars, &self.trans_func.code)
    }
    fn clock(&mut self, input: Value) -> Result<(), Error> {
        let mut vars = vec![(FIELD_IN.to_string(), input)];
        for (s, sm) in &self.states_machines {
            vars.push((s.to_string(), sm.get_otput()?));
        }
        let v = eval_to_val(&self.trans_func.mod_env, &vars, &self.trans_func.code)?;

        for (s, sm) in &mut self.states_machines {
            let Some(next_in) = v.get_field_of_value(s) else {
                bail!("field {s} not found")
            };
            sm.clock(next_in.clone())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IterModule {
    state_machine: GeneralModule,
    trans_func: CombExpVal,
    input_func: CombExpVal,
    otput_func: CombExpVal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IterModuleState {
    init_state_machine: GeneralModuleState,
    state_machines: Vec<GeneralModuleState>,
    trans_func: CodeEnv,
    input_func: CodeEnv,
    otput_func: CodeEnv,
}

const FIELD_NEXT: &str = "NEXT";
const FIELD_THIS: &str = "THIS";
const FILED_PREV: &str = "PREV";

impl IterModuleState {
    fn get_otput(&self) -> Result<Value, Error> {
        let o = self.state_machines[0].get_otput()?;
        eval_to_val(
            &self.otput_func.mod_env,
            &vec![(FIELD_STATE.to_string(), o)],
            &self.otput_func.code,
        )
    }
    fn clock(&mut self, input: Value) -> Result<(), Error> {
        let o_m1 = eval_to_val(
            &self.input_func.mod_env,
            &vec![(FIELD_IN.to_string(), input)],
            &self.input_func.code,
        )?;
        let otputs: Vec<Value> = self
            .state_machines
            .iter()
            .map(|sm| sm.get_otput())
            .collect::<Result<_, _>>()?;
        let o_other = self.init_state_machine.get_otput()?;
        let code_env = self.trans_func.clone();
        for (i, machine) in self.state_machines.iter_mut().enumerate() {
            let prev_out = if i == 0 {
                o_m1.clone()
            } else if i - 1 > otputs.len() {
                o_other.clone()
            } else {
                otputs[i - 1].clone()
            };
            let this_out = if i > otputs.len() {
                o_other.clone()
            } else {
                otputs[i].clone()
            };
            let next_out = if i + 1 > otputs.len() {
                o_other.clone()
            } else {
                otputs[i + 1].clone()
            };
            let input = eval_to_val(
                &code_env.mod_env,
                &vec![
                    (FILED_PREV.to_string(), prev_out),
                    (FIELD_THIS.to_string(), this_out),
                    (FILED_PREV.to_string(), next_out),
                ],
                &code_env.code,
            )?;
            machine.clock(input)?
        }
        Ok(())
    }
}
