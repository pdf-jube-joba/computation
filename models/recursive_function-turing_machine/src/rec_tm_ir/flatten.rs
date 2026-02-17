use std::collections::{HashMap, HashSet};

use super::machine::{validate_no_recursion, CallArg, Function, Program, Stmt};

pub fn flatten_program(program: &Program) -> Result<Program, String> {
    validate_no_recursion(program)?;
    let main = program
        .functions
        .get("main")
        .ok_or_else(|| "main() is not defined".to_string())?;
    let mut counter = 0usize;
    let mut functions = HashMap::new();
    let body = expand_stmts(&main.body, program, &mut counter)?;
    functions.insert(
        "main".to_string(),
        Function {
            name: "main".to_string(),
            params: main.params.clone(),
            body,
        },
    );
    Ok(Program {
        alphabet: program.alphabet.clone(),
        functions,
    })
}

fn expand_stmts(
    stmts: &[Stmt],
    program: &Program,
    counter: &mut usize,
) -> Result<Vec<Stmt>, String> {
    let mut expanded = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Loop { label, body } => {
                let body = expand_stmts(body, program, counter)?;
                expanded.push(Stmt::Loop {
                    label: label.clone(),
                    body,
                });
            }
            Stmt::Call { name, args } => {
                let mut call_expanded = expand_call(name, args, program, counter)?;
                expanded.append(&mut call_expanded);
            }
            _ => expanded.push(stmt.clone()),
        }
    }
    Ok(expanded)
}

fn expand_call(
    name: &str,
    args: &[CallArg],
    program: &Program,
    counter: &mut usize,
) -> Result<Vec<Stmt>, String> {
    let callee = program
        .functions
        .get(name)
        .ok_or_else(|| format!("Undefined function '{}'", name))?;
    if callee.params.len() != args.len() {
        return Err(format!(
            "Function '{}' expects {} args, got {}",
            name,
            callee.params.len(),
            args.len()
        ));
    }

    let suffix = *counter;
    *counter += 1;
    let shared_params: HashSet<String> = args
        .iter()
        .zip(callee.params.iter())
        .filter_map(|(arg, param)| if arg.shared { Some(param.clone()) } else { None })
        .collect();
    let mut var_map = build_var_map(callee, suffix, &shared_params);
    for (arg, param) in args.iter().zip(callee.params.iter()) {
        if arg.shared {
            var_map.insert(param.clone(), arg.name.clone());
        }
    }
    let label_map = build_label_map(&callee.body, suffix);
    let renamed = rename_stmts(&callee.body, &var_map, &label_map);
    let mut init = Vec::new();
    for (param, arg) in callee.params.iter().zip(args.iter()) {
        if arg.shared {
            continue;
        }
        let new_param = var_map
            .get(param)
            .cloned()
            .unwrap_or_else(|| param.clone());
        init.push(Stmt::Assign(new_param, arg.name.clone()));
    }
    let mut body = expand_stmts(&renamed, program, counter)?;
    init.append(&mut body);
    Ok(init)
}

fn build_var_map(
    func: &Function,
    suffix: usize,
    shared_params: &HashSet<String>,
) -> HashMap<String, String> {
    let mut vars = HashSet::new();
    for param in &func.params {
        vars.insert(param.clone());
    }
    collect_vars(&func.body, &mut vars);
    vars.into_iter()
        .map(|var| {
            if shared_params.contains(&var) {
                (var.clone(), var)
            } else {
                let renamed = format!("__flat{}_{}", suffix, var);
                (var, renamed)
            }
        })
        .collect()
}

fn build_label_map(stmts: &[Stmt], suffix: usize) -> HashMap<String, String> {
    let mut labels = HashSet::new();
    collect_labels(stmts, &mut labels);
    labels
        .into_iter()
        .map(|label| {
            let renamed = format!("__flat{}_{}", suffix, label);
            (label, renamed)
        })
        .collect()
}

fn rename_stmts(
    stmts: &[Stmt],
    var_map: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Vec<Stmt> {
    stmts
        .iter()
        .map(|stmt| rename_stmt(stmt, var_map, label_map))
        .collect()
}

fn rename_var(var: &str, var_map: &HashMap<String, String>) -> String {
    var_map
        .get(var)
        .cloned()
        .unwrap_or_else(|| var.to_string())
}

fn rename_label(label: &str, label_map: &HashMap<String, String>) -> String {
    label_map
        .get(label)
        .cloned()
        .unwrap_or_else(|| label.to_string())
}

fn rename_stmt(
    stmt: &Stmt,
    var_map: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Stmt {
    match stmt {
        Stmt::Lt => Stmt::Lt,
        Stmt::Rt => Stmt::Rt,
        Stmt::Read(var) => Stmt::Read(rename_var(var, var_map)),
        Stmt::Stor(var) => Stmt::Stor(rename_var(var, var_map)),
        Stmt::Assign(dst, src) => Stmt::Assign(rename_var(dst, var_map), rename_var(src, var_map)),
        Stmt::IfBreak { var, value, label } => Stmt::IfBreak {
            var: rename_var(var, var_map),
            value: value.clone(),
            label: rename_label(label, label_map),
        },
        Stmt::Loop { label, body } => Stmt::Loop {
            label: rename_label(label, label_map),
            body: rename_stmts(body, var_map, label_map),
        },
        Stmt::Call { name, args } => Stmt::Call {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| CallArg {
                    shared: arg.shared,
                    name: rename_var(&arg.name, var_map),
                })
                .collect(),
        },
    }
}

fn collect_vars(stmts: &[Stmt], set: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Read(var) | Stmt::Stor(var) | Stmt::IfBreak { var, .. } => {
                set.insert(var.clone());
            }
            Stmt::Assign(dst, src) => {
                set.insert(dst.clone());
                set.insert(src.clone());
            }
            Stmt::Loop { body, .. } => collect_vars(body, set),
            Stmt::Call { args, .. } => {
                for arg in args {
                    set.insert(arg.name.clone());
                }
            }
            Stmt::Lt | Stmt::Rt => {}
        }
    }
}

fn collect_labels(stmts: &[Stmt], set: &mut HashSet<String>) {
    for stmt in stmts {
        if let Stmt::Loop { label, body } = stmt {
            set.insert(label.clone());
            collect_labels(body, set);
        }
    }
}
