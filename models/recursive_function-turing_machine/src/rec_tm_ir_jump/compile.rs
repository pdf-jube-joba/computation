use utils::{Compiler, Machine, TextCodec};

use crate::rec_tm_ir::{flatten_program, Program as Ir1Program, Stmt as Ir1Stmt};

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
        .get("main")
        .ok_or_else(|| "main() is not defined".to_string())?;
    let body = compile_block(&main.body)?;
    Ok(Ir2Program {
        alphabet: program.alphabet.clone(),
        body,
    })
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
                        if let Ir2Stmt::JumpIf { target, .. } = &mut instrs[fixup] {
                            *target = end;
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
