use serde::Serialize;
use wasm_bindgen::JsValue;

pub mod alphabet;
pub mod bool;
pub mod number;
pub mod set;
pub mod variable;

// This trait is implemented by concrete models. It stays generic and not object-safe.
pub trait MealyMachine: Sized {
    type Input: Serialize;
    type Output: Serialize;
    type This: Serialize;

    fn parse_self(input: &str) -> Result<Self, String>;
    fn parse_input(input: &str) -> Result<Self::Input, String>;
    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String>;
    fn current(&self) -> Self::This;
}

pub trait OneTime: Sized {
    type Code: Serialize + Clone; // (A)
    type Input: Serialize;
    type Env: Serialize;

    fn parse_code(input: &str) -> Result<Self::Code, String>;
    fn parse_input(input: &str) -> Result<Self::Input, String>;
    fn setup(code: Self::Code, input: Self::Input) -> Result<Self, String>;
    fn run_onestep(&mut self);
    fn is_terminated(&self) -> bool;
    fn current_env(&self) -> Self::Env;
}

#[derive(Debug, Clone, Serialize)]
pub enum OneTimeMachine<T>
where
    T: OneTime,
{
    Machine(T),
    Code(T::Code),
}

#[derive(Debug, Clone, Serialize)]
pub enum OneTimeInput<T>
where
    T: OneTime,
{
    Input(T::Input),
    Otherwise,
}

impl<T> MealyMachine for OneTimeMachine<T>
where
    T: OneTime,
{
    type Input = T::Input;
    type Output = bool;
    type This = T::Env;
    fn parse_self(input: &str) -> Result<Self, String> {
        let code = T::parse_code(input)?;
        Ok(OneTimeMachine::Code(code))
    }
    fn parse_input(input: &str) -> Result<Self::Input, String> {
        T::parse_input(input)
    }
    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String> {
        match self {
            OneTimeMachine::Code(code) => {
                // cannot move out of code, so clone it ... (A)
                *self = OneTimeMachine::Machine(T::setup(code.clone(), input)?);
                Ok(None)
            }
            OneTimeMachine::Machine(machine) => {
                machine.run_onestep();
                if machine.is_terminated() {
                    Ok(Some(true))
                } else {
                    Ok(Some(false))
                }
            }
        }
    }
    fn current(&self) -> Self::This {
        match self {
            OneTimeMachine::Code(_) => panic!("Machine not initialized"),
            OneTimeMachine::Machine(machine) => machine.current_env(),
        }
    }
}

// Object-safe wrapper used at runtime.
pub trait WebView {
    fn step(&mut self, input: &str) -> Result<Option<JsValue>, String>;
    fn current(&self) -> JsValue;
}

pub trait ToJsResult<T> {
    fn to_js(self) -> Result<T, String>;
}

impl<T> ToJsResult<T> for anyhow::Result<T> {
    fn to_js(self) -> Result<T, String> {
        self.map_err(|e| format!("{e:?}"))
    }
}

impl<T> WebView for T
where
    T: MealyMachine,
{
    fn step(&mut self, input: &str) -> Result<Option<JsValue>, String> {
        let parsed = <Self as MealyMachine>::parse_input(input)?;
        let output = <Self as MealyMachine>::step(self, parsed)?;
        match output {
            Some(o) => {
                let js = serde_wasm_bindgen::to_value(&o).map_err(|e| e.to_string())?;
                Ok(Some(js))
            }
            None => Ok(None),
        }
    }

    fn current(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&<Self as MealyMachine>::current(self))
            .unwrap_or_else(|e| JsValue::from_str(&e.to_string()))
    }
}
