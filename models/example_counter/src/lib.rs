use serde_json::json;
use utils::{Compiler, Machine, StepResult, TextCodec};

#[derive(Clone)]
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

impl From<Counter> for serde_json::Value {
    fn from(counter: Counter) -> Self {
        json!([{ "kind": "text", "text": counter.count.to_string() }])
    }
}

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
    type ROutput = String;
    type FOutput = String;
    type SnapShot = Counter;

    fn make(code: Self::Code, _ainput: Self::AInput) -> Result<Self, String> {
        Ok(code)
    }

    fn step(self, input: Self::RInput) -> Result<StepResult<Self>, String> {
        let mut next = self;
        match input {
            Command::Increment => {
                next.count += 1;
                if next.count >= 10 {
                    let snapshot = next.clone();
                    Ok(StepResult::Halt {
                        snapshot,
                        output: "End".to_string(),
                    })
                } else {
                    Ok(StepResult::Continue {
                        next,
                        output: "inc".to_string(),
                    })
                }
            }
            Command::Decrement => {
                if next.count == 0 {
                    Err("Count cannot be negative".to_string())
                } else {
                    next.count -= 1;
                    Ok(StepResult::Continue {
                        next,
                        output: "dec".to_string(),
                    })
                }
            }
        }
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}

pub struct ExampleCounterCompiler;

impl Compiler for ExampleCounterCompiler {
    type Source = Counter;
    type Target = Counter;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        Ok(source)
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
        Ok(ainput)
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
        Ok(rinput)
    }

    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String> {
        Ok(output)
    }

    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String> {
        Ok(output)
    }
}
