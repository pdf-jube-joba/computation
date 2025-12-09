use serde::Serialize;

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
    type Input: Serialize + Default;
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
