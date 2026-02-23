use std::collections::{HashMap, HashSet};

use utils::{Compiler, Machine};

use crate::rec_tm_ir::{
    Block, Condition as Ir1Condition, Function, LValue as Ir1LValue, Program as Ir1Program,
    RValue as Ir1RValue, Stmt as Ir1Stmt, validate_no_recursion,
};

use crate::rec_tm_ir_jump::{
    Condition as Ir2Condition, LValue as Ir2LValue, Program as Ir2Program, RValue as Ir2RValue,
    Stmt as Ir2Stmt,
};

pub struct RecTmIrToJumpCompiler;

impl Compiler for RecTmIrToJumpCompiler {
    type Source = crate::rec_tm_ir::RecTmIrMachine;
    type Target = crate::rec_tm_ir_jump::RecTmIrJumpMachine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        compile_to_jump(&source)
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

    fn decode_routput(
        output: <<Self as Compiler>::Target as Machine>::ROutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::ROutput, String> {
        let _: () = output;
        Ok(())
    }

    fn decode_foutput(
        output: <<Self as Compiler>::Target as Machine>::FOutput,
    ) -> Result<<<Self as Compiler>::Source as Machine>::FOutput, String> {
        Ok(output)
    }
}

pub fn compile_to_jump(program: &Ir1Program) -> Result<Ir2Program, String> {
    let flat = flatten_program(program)?;
    let main = flat
        .functions
        .iter()
        .find(|func| func.name == "main")
        .ok_or_else(|| "main() is not defined".to_string())?;
    let body = compile_blocks(&main.blocks)?;
    Ok(Ir2Program {
        alphabet: flat.alphabet.clone(),
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
    let blocks = expand_blocks(&main.blocks, program, &mut counter)?;
    Ok(Ir1Program {
        alphabet: program.alphabet.clone(),
        functions: vec![Function {
            name: "main".to_string(),
            blocks,
        }
        .into()],
    })
}

fn expand_blocks(
    blocks: &[Block],
    program: &Ir1Program,
    counter: &mut usize,
) -> Result<Vec<Block>, String> {
    let mut expanded = Vec::new();
    for block in blocks {
        expanded.extend(expand_block(block, program, counter)?);
    }
    Ok(expanded)
}

fn expand_block(
    block: &Block,
    program: &Ir1Program,
    counter: &mut usize,
) -> Result<Vec<Block>, String> {
    let mut prefix = Vec::new();
    for (idx, stmt) in block.body.iter().enumerate() {
        match stmt {
            Ir1Stmt::Call { func } => {
                let suffix = *counter;
                *counter += 1;
                let cont_label = format!("__cont_{}_{}", block.label, suffix);
                let callee_blocks = expand_call(func, program, counter, &cont_label, suffix)?;

                let mut out = Vec::new();
                out.push(Block {
                    label: block.label.clone(),
                    body: prefix,
                });
                out.extend(callee_blocks);

                let cont_block = Block {
                    label: cont_label,
                    body: block.body[idx + 1..].to_vec(),
                };
                out.extend(expand_block(&cont_block, program, counter)?);
                return Ok(out);
            }
            _ => prefix.push(stmt.clone()),
        }
    }

    Ok(vec![Block {
        label: block.label.clone(),
        body: prefix,
    }])
}

fn expand_call(
    func: &std::rc::Rc<Function>,
    program: &Ir1Program,
    counter: &mut usize,
    return_label: &str,
    suffix: usize,
) -> Result<Vec<Block>, String> {
    let callee = program
        .functions
        .iter()
        .find(|f| f.name == func.name)
        .ok_or_else(|| format!("Undefined function '{}'", func.name))?;
    let var_map = build_var_map(callee, suffix);
    let label_map = build_label_map(&callee.blocks, suffix);
    let renamed = rename_blocks(&callee.blocks, &var_map, &label_map);
    let replaced = replace_returns(&renamed, return_label);
    expand_blocks(&replaced, program, counter)
}

fn build_var_map(func: &Function, suffix: usize) -> HashMap<String, String> {
    let mut vars = HashSet::new();
    collect_vars(&func.blocks, &mut vars);
    vars.into_iter()
        .map(|var| (var.clone(), format!("__flat{}_{}", suffix, var)))
        .collect()
}

fn build_label_map(blocks: &[Block], suffix: usize) -> HashMap<String, String> {
    let mut labels = HashSet::new();
    collect_labels(blocks, &mut labels);
    labels
        .into_iter()
        .map(|label| (label.clone(), format!("__flat{}_{}", suffix, label)))
        .collect()
}

fn rename_blocks(
    blocks: &[Block],
    var_map: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Vec<Block> {
    blocks
        .iter()
        .map(|block| Block {
            label: rename_label(&block.label, label_map),
            body: block
                .body
                .iter()
                .map(|stmt| rename_stmt(stmt, var_map, label_map))
                .collect(),
        })
        .collect()
}

fn rename_var(var: &str, var_map: &HashMap<String, String>) -> String {
    var_map.get(var).cloned().unwrap_or_else(|| var.to_string())
}

fn rename_label(label: &str, label_map: &HashMap<String, String>) -> String {
    label_map
        .get(label)
        .cloned()
        .unwrap_or_else(|| label.to_string())
}

fn rename_lvalue(value: &Ir1LValue, var_map: &HashMap<String, String>) -> Ir1LValue {
    match value {
        Ir1LValue::Var(var) => Ir1LValue::Var(rename_var(var, var_map)),
        Ir1LValue::Head => Ir1LValue::Head,
    }
}

fn rename_rvalue(value: &Ir1RValue, var_map: &HashMap<String, String>) -> Ir1RValue {
    match value {
        Ir1RValue::Var(var) => Ir1RValue::Var(rename_var(var, var_map)),
        Ir1RValue::Head => Ir1RValue::Head,
        Ir1RValue::Const(sign) => Ir1RValue::Const(sign.clone()),
    }
}

fn rename_condition(
    cond: &Option<Ir1Condition>,
    var_map: &HashMap<String, String>,
) -> Option<Ir1Condition> {
    cond.as_ref().map(|cond| Ir1Condition {
        left: rename_rvalue(&cond.left, var_map),
        right: rename_rvalue(&cond.right, var_map),
    })
}

fn rename_stmt(
    stmt: &Ir1Stmt,
    var_map: &HashMap<String, String>,
    label_map: &HashMap<String, String>,
) -> Ir1Stmt {
    match stmt {
        Ir1Stmt::Lt => Ir1Stmt::Lt,
        Ir1Stmt::Rt => Ir1Stmt::Rt,
        Ir1Stmt::Assign { dst, src } => Ir1Stmt::Assign {
            dst: rename_lvalue(dst, var_map),
            src: rename_rvalue(src, var_map),
        },
        Ir1Stmt::Break { cond } => Ir1Stmt::Break {
            cond: rename_condition(cond, var_map),
        },
        Ir1Stmt::Continue { cond } => Ir1Stmt::Continue {
            cond: rename_condition(cond, var_map),
        },
        Ir1Stmt::Jump { label, cond } => Ir1Stmt::Jump {
            label: rename_label(label, label_map),
            cond: rename_condition(cond, var_map),
        },
        Ir1Stmt::Return { cond } => Ir1Stmt::Return {
            cond: rename_condition(cond, var_map),
        },
        Ir1Stmt::Call { func } => Ir1Stmt::Call { func: func.clone() },
    }
}

fn replace_returns(blocks: &[Block], label: &str) -> Vec<Block> {
    blocks
        .iter()
        .map(|block| Block {
            label: block.label.clone(),
            body: block
                .body
                .iter()
                .map(|stmt| match stmt {
                    Ir1Stmt::Return { cond } => Ir1Stmt::Jump {
                        label: label.to_string(),
                        cond: cond.clone(),
                    },
                    _ => stmt.clone(),
                })
                .collect(),
        })
        .collect()
}

fn collect_vars(blocks: &[Block], set: &mut HashSet<String>) {
    for block in blocks {
        for stmt in &block.body {
            match stmt {
                Ir1Stmt::Assign { dst, src } => {
                    if let Ir1LValue::Var(var) = dst {
                        set.insert(var.clone());
                    }
                    if let Ir1RValue::Var(var) = src {
                        set.insert(var.clone());
                    }
                }
                Ir1Stmt::Break { cond }
                | Ir1Stmt::Continue { cond }
                | Ir1Stmt::Jump { cond, .. }
                | Ir1Stmt::Return { cond } => {
                    if let Some(cond) = cond {
                        if let Ir1RValue::Var(var) = &cond.left {
                            set.insert(var.clone());
                        }
                        if let Ir1RValue::Var(var) = &cond.right {
                            set.insert(var.clone());
                        }
                    }
                }
                Ir1Stmt::Call { .. } | Ir1Stmt::Lt | Ir1Stmt::Rt => {}
            }
        }
    }
}

fn collect_labels(blocks: &[Block], set: &mut HashSet<String>) {
    for block in blocks {
        set.insert(block.label.clone());
    }
}

fn compile_blocks(blocks: &[Block]) -> Result<Vec<Ir2Stmt>, String> {
    let mut labels = HashMap::new();
    let mut lowered = Vec::new();

    for (idx, block) in blocks.iter().enumerate() {
        if labels.insert(block.label.clone(), lowered.len()).is_some() {
            return Err(format!("label '{}' is duplicated", block.label));
        }
        let next_label = blocks.get(idx + 1).map(|b| b.label.clone());

        for stmt in &block.body {
            match stmt {
                Ir1Stmt::Break { cond } => {
                    if let Some(label) = &next_label {
                        lowered.push(Ir1Stmt::Jump {
                            label: label.clone(),
                            cond: cond.clone(),
                        });
                    } else {
                        lowered.push(Ir1Stmt::Return { cond: cond.clone() });
                    }
                }
                Ir1Stmt::Continue { cond } => {
                    lowered.push(Ir1Stmt::Jump {
                        label: block.label.clone(),
                        cond: cond.clone(),
                    });
                }
                _ => lowered.push(stmt.clone()),
            }
        }
    }

    compile_flat_block(&lowered, &labels)
}

fn to_ir2_lvalue(value: &Ir1LValue) -> Ir2LValue {
    match value {
        Ir1LValue::Var(var) => Ir2LValue::Var(var.clone()),
        Ir1LValue::Head => Ir2LValue::Head,
    }
}

fn to_ir2_rvalue(value: &Ir1RValue) -> Ir2RValue {
    match value {
        Ir1RValue::Var(var) => Ir2RValue::Var(var.clone()),
        Ir1RValue::Head => Ir2RValue::Head,
        Ir1RValue::Const(sign) => Ir2RValue::Const(sign.clone()),
    }
}

fn to_ir2_condition(cond: &Option<Ir1Condition>) -> Option<Ir2Condition> {
    cond.as_ref().map(|cond| Ir2Condition {
        left: to_ir2_rvalue(&cond.left),
        right: to_ir2_rvalue(&cond.right),
    })
}

fn compile_flat_block(stmts: &[Ir1Stmt], labels: &HashMap<String, usize>) -> Result<Vec<Ir2Stmt>, String> {
    let mut instrs = Vec::new();
    let mut return_fixups = Vec::new();

    for stmt in stmts {
        match stmt {
            Ir1Stmt::Lt => instrs.push(Ir2Stmt::Lt),
            Ir1Stmt::Rt => instrs.push(Ir2Stmt::Rt),
            Ir1Stmt::Assign { dst, src } => instrs.push(Ir2Stmt::Assign {
                dst: to_ir2_lvalue(dst),
                src: to_ir2_rvalue(src),
            }),
            Ir1Stmt::Jump { label, cond } => {
                let target = labels
                    .get(label)
                    .copied()
                    .ok_or_else(|| format!("jump target '{}' not found", label))?;
                instrs.push(Ir2Stmt::Jump {
                    target,
                    cond: to_ir2_condition(cond),
                });
            }
            Ir1Stmt::Return { cond } => {
                return_fixups.push(instrs.len());
                instrs.push(Ir2Stmt::Jump {
                    target: 0,
                    cond: to_ir2_condition(cond),
                });
            }
            Ir1Stmt::Break { .. } | Ir1Stmt::Continue { .. } => {
                return Err("break/continue should have been lowered".to_string());
            }
            Ir1Stmt::Call { func } => {
                return Err(format!("call '{}' should have been flattened", func.name));
            }
        }
    }

    let end = instrs.len();
    for idx in return_fixups {
        if let Ir2Stmt::Jump { target, .. } = &mut instrs[idx] {
            *target = end;
        }
    }
    Ok(instrs)
}
