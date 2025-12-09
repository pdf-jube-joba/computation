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
    fn get_code(&self) -> Self::Code;
}

impl<T> MealyMachine for T
where
    T: OneTime,
{
    type Input = Option<T::Input>;
    type Output = bool;
    type This = T::Env;
    fn parse_self(input: &str) -> Result<Self, String> {
        let code = T::parse_code(input)?;
        T::setup(code, T::Input::default())
    }
    fn parse_input(input: &str) -> Result<Self::Input, String> {
        if input.trim().is_empty() {
            Ok(None)
        } else {
            T::parse_input(input).map(Some)
        }
    }
    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String> {
        match input {
            Some(input) => {
                let code = self.get_code();
                *self = T::setup(code, input)?;
                Ok(None)
            }
            None => {
                if self.is_terminated() {
                    Ok(Some(true))
                } else {
                    self.run_onestep();
                    if self.is_terminated() {
                        Ok(Some(true))
                    } else {
                        Ok(Some(false))
                    }
                }
            }
        }
    }
    fn current(&self) -> Self::This {
        self.current_env()
    }
}
