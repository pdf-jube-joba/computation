use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::machine::{self, RecursiveFunctions};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Proj {
    length: usize,
    number: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Comp {
    length: usize,
    inner: Vec<Function>,
    outer: Box<Function>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Prim {
    zero: Box<Function>,
    succ: Box<Function>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Muop {
    muop: Box<Function>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Function {
    Zero,
    Succ,
    Proj(Proj),
    Comp(Comp),
    Prim(Prim),
    Muop(Muop),
    Exist(String),
}

fn convert(
    func: Function,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<RecursiveFunctions, String> {
    match func {
        Function::Zero => Ok(RecursiveFunctions::zero()),
        Function::Succ => Ok(RecursiveFunctions::succ()),
        Function::Proj(Proj { length, number }) => RecursiveFunctions::projection(length, number),
        Function::Comp(Comp {
            length,
            inner,
            outer,
        }) => {
            let inner = inner
                .into_iter()
                .map(|func| convert(func, map))
                .collect::<Result<_, _>>();
            let outer = convert(*outer, map);
            RecursiveFunctions::composition(length, inner?, outer?)
        }
        Function::Prim(Prim { zero, succ }) => {
            let zero = convert(*zero, map);
            let succ = convert(*succ, map);
            RecursiveFunctions::primitive_recursion(zero?, succ?)
        }
        Function::Muop(Muop { muop }) => {
            let muop = convert(*muop, map);
            RecursiveFunctions::muoperator(muop?)
        }
        Function::Exist(string) => map
            .get(&string)
            .cloned()
            .ok_or_else(|| format!("not found function name: {string}")),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionData {
    name: String,
    function: Function,
}

pub fn parse(str: &str) -> Result<machine::RecursiveFunctions, String> {
    let funcs_data: Vec<FunctionData> =
        serde_json::from_str(str).map_err(|err| format!("{err:?}"))?;
    let mut map: HashMap<String, RecursiveFunctions> = HashMap::new();
    for FunctionData { name, function } in funcs_data {
        let func = convert(function, &map)?;
        map.insert(name, func);
    }
    map.get("main")
        .cloned()
        .ok_or("main is not found".to_string())
}

#[cfg(test)]
mod tests {
    use crate::machine::interpreter;
    use utils::number::Number;

    use super::*;
    #[test]
    fn json_test() {
        let stru = Function::Zero;
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");

        let stru = Function::Succ;
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");

        let stru = Function::Proj(Proj {
            length: 3,
            number: 0,
        });
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");

        let stru = Function::Comp(Comp {
            length: 1,
            inner: vec![Function::Zero],
            outer: Box::new(Function::Zero),
        });
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");

        let stru = Function::Prim(Prim {
            zero: Box::new(Function::Zero),
            succ: Box::new(Function::Zero),
        });
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");

        let stru = Function::Muop(Muop {
            muop: Box::new(Function::Zero),
        });
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");

        let stru = Function::Exist("add".to_string());
        let json = serde_json::to_string(&stru).unwrap();
        println!("{json}");
    }
    #[test]
    fn json_test_2() {
        let func_data: Vec<FunctionData> = vec![FunctionData {
            name: "add1".to_string(),
            function: Function::Succ,
        }];
        let str = serde_json::to_string(&func_data).unwrap();
        println!("{str}");
        let json: serde_json::Value = serde_json::from_str(&str).unwrap();
        let func_data_from: Vec<FunctionData> = serde_json::from_value(json).unwrap();
        assert_eq!(func_data, func_data_from)
    }
    #[test]
    fn json_test_3() {
        let func_str = r#"[{"name":"main","function":"Succ"}]"#;
        let _ = parse(func_str).unwrap();
        let func_str = r#"
[
    {
        "name": "(x,y,z) -> succ x",
        "function": {
            "Comp": {
                "length": 3,
                "inner": [
                    {"Proj": {"length": 3, "number": 0}}
                ],
                "outer": "Succ"
            }
        }
    },
    {
        "name": "id",
        "function": {
            "Proj": {
                "length": 1,
                "number": 0
            }
        }
    },
    {
        "name": "main",
        "function": {
            "Prim": {
                "zero": {"Exist": "id"},
                "succ": {"Exist": "(x,y,z) -> succ x"}
            }
        }
    }
]"#;
        let func = parse(func_str).unwrap();
        let res = interpreter(&func).unchecked_subst(vec![1, 2].into());
        assert_eq!(Number::from(3), res)
    }
}
