use utils::{Compiler, Machine, TextCodec};

use std::collections::{HashMap, HashSet};

use crate::rec_tm_ir::{
    validate_no_recursion, CallArg, Function, Program as Ir1Program, Stmt as Ir1Stmt,
};

use super::machine::{Program as Ir2Program, Stmt as Ir2Stmt};

pub struct RecTmIrToJumpCompiler;

impl Compiler for RecTmIrToJumpCompiler {
    type Source = crate::rec_tm_ir::RecTmIrMachine;
    type Target = crate::rec_tm_ir_jump::RecTmIrJumpMachine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        let flattened = flatten_program(&source)?;
        compile_to_jump(&flattened)
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
        Ok(ainput)
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
        let _: () = rinput;
        Ok(())
    }

    fn decode_output(
        output: <<Self as Compiler>::Target as Machine>::Output,
    ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String> {
        let text = output.print();
        <crate::rec_tm_ir::Environment as TextCodec>::parse(&text)
    }
}

fn compile_to_jump(program: &Ir1Program) -> Result<Ir2Program, String> {
    let main = program
        .functions
        .iter()
        .find(|func| func.name == "main")
        .ok_or_else(|| "main() is not defined".to_string())?;
    let body = compile_block(&main.body)?;
    Ok(Ir2Program {
        alphabet: program.alphabet.clone(),
        body,
    })
}

pub(crate) fn flatten_program(program: &Ir1Program) -> Result<Ir1Program, String> {
    validate_no_recursion(program)?;
    let main = program
        .functions
        .iter()
        .find(|func| func.name == "main")
        .ok_or_else(|| "main() is not defined".to_string())?;
    let mut counter = 0usize;
    let mut functions: Vec<Function> = Vec::new();
    let body = expand_stmts(&main.body, program, &mut counter)?;
    functions.push(Function {
        name: "main".to_string(),
        params: main.params.clone(),
        body,
    });
    Ok(Ir1Program {
        alphabet: program.alphabet.clone(),
        functions,
    })
}

fn expand_stmts(
    stmts: &[Ir1Stmt],
    program: &Ir1Program,
    counter: &mut usize,
) -> Result<Vec<Ir1Stmt>, String> {
    let mut expanded = Vec::new();
    for stmt in stmts {
        match stmt {
            Ir1Stmt::Loop { label, body } => {
                let body = expand_stmts(body, program, counter)?;
                expanded.push(Ir1Stmt::Loop {
                    label: label.clone(),
                    body,
                });
            }
            Ir1Stmt::Call { name, args } => {
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
    program: &Ir1Program,
    counter: &mut usize,
) -> Result<Vec<Ir1Stmt>, String> {
    let callee = program
        .functions
        .iter()
        .find(|func| func.name == name)
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
        init.push(Ir1Stmt::Assign(new_param, arg.name.clone()));
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

fn build_label_map(stmts: &[Ir1Stmt], suffix: usize) -> HashMap<String, String> {
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
    stmts: &[Ir1Stmt],
    var_map: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Vec<Ir1Stmt> {
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
    stmt: &Ir1Stmt,
    var_map: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Ir1Stmt {
    match stmt {
        Ir1Stmt::Lt => Ir1Stmt::Lt,
        Ir1Stmt::Rt => Ir1Stmt::Rt,
        Ir1Stmt::Read(var) => Ir1Stmt::Read(rename_var(var, var_map)),
        Ir1Stmt::Stor(var) => Ir1Stmt::Stor(rename_var(var, var_map)),
        Ir1Stmt::StorConst(value) => Ir1Stmt::StorConst(value.clone()),
        Ir1Stmt::Assign(dst, src) => {
            Ir1Stmt::Assign(rename_var(dst, var_map), rename_var(src, var_map))
        }
        Ir1Stmt::ConstAssign(dst, value) => {
            Ir1Stmt::ConstAssign(rename_var(dst, var_map), value.clone())
        }
        Ir1Stmt::IfBreak { var, value, label } => Ir1Stmt::IfBreak {
            var: rename_var(var, var_map),
            value: value.clone(),
            label: rename_label(label, label_map),
        },
        Ir1Stmt::IfBreakHead { value, label } => Ir1Stmt::IfBreakHead {
            value: value.clone(),
            label: rename_label(label, label_map),
        },
        Ir1Stmt::Loop { label, body } => Ir1Stmt::Loop {
            label: rename_label(label, label_map),
            body: rename_stmts(body, var_map, label_map),
        },
        Ir1Stmt::Call { name, args } => Ir1Stmt::Call {
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

fn collect_vars(stmts: &[Ir1Stmt], set: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Ir1Stmt::Read(var) | Ir1Stmt::Stor(var) | Ir1Stmt::IfBreak { var, .. } => {
                set.insert(var.clone());
            }
            Ir1Stmt::Assign(dst, src) => {
                set.insert(dst.clone());
                set.insert(src.clone());
            }
            Ir1Stmt::ConstAssign(dst, _) => {
                set.insert(dst.clone());
            }
            Ir1Stmt::Loop { body, .. } => collect_vars(body, set),
            Ir1Stmt::Call { args, .. } => {
                for arg in args {
                    set.insert(arg.name.clone());
                }
            }
            Ir1Stmt::Lt | Ir1Stmt::Rt => {}
            Ir1Stmt::StorConst(_) => {}
            Ir1Stmt::IfBreakHead { .. } => {}
        }
    }
}

fn collect_labels(stmts: &[Ir1Stmt], set: &mut HashSet<String>) {
    for stmt in stmts {
        if let Ir1Stmt::Loop { label, body } = stmt {
            set.insert(label.clone());
            collect_labels(body, set);
        }
    }
}

struct LoopContext {
    label: String,
    start: usize,
    break_fixups: Vec<usize>,
}

fn compile_block(stmts: &[Ir1Stmt]) -> Result<Vec<Ir2Stmt>, String> {
    let mut instrs = Vec::new();
    let mut loop_stack = Vec::new();
    compile_block_inner(stmts, &mut instrs, &mut loop_stack)?;
    if !loop_stack.is_empty() {
        return Err("Loop stack not empty after compilation".to_string());
    }
    Ok(instrs)
}

fn compile_block_inner(
    stmts: &[Ir1Stmt],
    instrs: &mut Vec<Ir2Stmt>,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            Ir1Stmt::Lt => instrs.push(Ir2Stmt::Lt),
            Ir1Stmt::Rt => instrs.push(Ir2Stmt::Rt),
            Ir1Stmt::Read(var) => instrs.push(Ir2Stmt::Read(var.clone())),
            Ir1Stmt::Stor(var) => instrs.push(Ir2Stmt::Stor(var.clone())),
            Ir1Stmt::StorConst(value) => instrs.push(Ir2Stmt::StorConst(value.clone())),
            Ir1Stmt::Assign(dst, src) => instrs.push(Ir2Stmt::Assign(dst.clone(), src.clone())),
            Ir1Stmt::ConstAssign(dst, value) => {
                instrs.push(Ir2Stmt::ConstAssign(dst.clone(), value.clone()));
            }
            Ir1Stmt::IfBreak { var, value, label } => {
                let mut found = None;
                for (idx, ctx) in loop_stack.iter().enumerate().rev() {
                    if ctx.label == *label {
                        found = Some(idx);
                        break;
                    }
                }
                let Some(loop_index) = found else {
                    return Err(format!("break target '{}' not found", label));
                };
                let index = instrs.len();
                instrs.push(Ir2Stmt::JumpIf {
                    var: var.clone(),
                    value: value.clone(),
                    target: 0,
                });
                loop_stack[loop_index].break_fixups.push(index);
            }
            Ir1Stmt::IfBreakHead { value, label } => {
                let mut found = None;
                for (idx, ctx) in loop_stack.iter().enumerate().rev() {
                    if ctx.label == *label {
                        found = Some(idx);
                        break;
                    }
                }
                let Some(loop_index) = found else {
                    return Err(format!("break target '{}' not found", label));
                };
                let index = instrs.len();
                instrs.push(Ir2Stmt::JumpIfHead {
                    value: value.clone(),
                    target: 0,
                });
                loop_stack[loop_index].break_fixups.push(index);
            }
            Ir1Stmt::Loop { label, body } => {
                if loop_stack.iter().any(|ctx| ctx.label == *label) {
                    return Err(format!("Loop label '{}' is duplicated", label));
                }
                let start = instrs.len();
                loop_stack.push(LoopContext {
                    label: label.clone(),
                    start,
                    break_fixups: Vec::new(),
                });
                compile_block_inner(body, instrs, loop_stack)?;
                instrs.push(Ir2Stmt::Jump(start));
                let end = instrs.len();
                if let Some(ctx) = loop_stack.pop() {
                    for fixup in ctx.break_fixups {
                        match &mut instrs[fixup] {
                            Ir2Stmt::JumpIf { target, .. } => {
                                *target = end;
                            }
                            Ir2Stmt::JumpIfHead { target, .. } => {
                                *target = end;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ir1Stmt::Call { name, .. } => {
                return Err(format!("call '{}' should have been flattened", name));
            }
        }
    }
    Ok(())
}
