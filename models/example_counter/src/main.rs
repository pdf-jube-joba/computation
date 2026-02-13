use serde::Serialize;
use utils::{Machine, TextCodec};

#[derive(Clone, Serialize)]
pub struct Counter {
    pub count: usize,
}

impl TextCodec for Counter {
    fn parse(text: &str) -> Result<Self, String> {
        let counter: usize = if text.trim().is_empty() {
            0
        } else {
            text.trim().parse::<usize>().map_err(|e| e.to_string())?
        };
        Ok(Counter { count: counter })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.count)
    }
}

#[derive(Serialize)]
pub enum Command {
    Increment,
    Decrement,
}

impl TextCodec for Command {
    fn parse(text: &str) -> Result<Self, String> {
        match text.trim() {
            "inc" => Ok(Command::Increment),
            "dec" => Ok(Command::Decrement),
            _ => Err("Invalid command".to_string()),
        }
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Command::Increment => write!(f, "inc"),
            Command::Decrement => write!(f, "dec"),
        }
    }
}

impl Machine for Counter {
    type Code = Counter;
    type AInput = ();
    type RInput = Command;
    type Output = String;
    type SnapShot = Counter;

    fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
        Ok(code)
    }

    fn step(&mut self, input: Self::RInput) -> Result<Option<Self::Output>, String> {
        match input {
            Command::Increment => {
                self.count += 1;
                if self.count >= 10 {
                    Ok(Some("End".to_string()))
                } else {
                    Ok(None)
                }
            }
            Command::Decrement => {
                if self.count == 0 {
                    Err("Count cannot be negative".to_string())
                } else {
                    self.count -= 1;
                    Ok(None)
                }
            }
        }
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}

utils::web_model!(Counter);
