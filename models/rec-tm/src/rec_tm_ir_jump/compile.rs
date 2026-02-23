// use utils::{Compiler, Machine, TextCodec};

// use std::collections::{HashMap, HashSet};

// use crate::rec_tm_ir::{
//     Block, Function, Program as Ir1Program, Stmt as Ir1Stmt, validate_no_recursion,
// };

// use super::machine::{Program as Ir2Program, Stmt as Ir2Stmt};

// pub struct RecTmIrToJumpCompiler;

// impl Compiler for RecTmIrToJumpCompiler {
//     type Source = crate::rec_tm_ir::RecTmIrMachine;
//     type Target = crate::rec_tm_ir_jump::RecTmIrJumpMachine;

//     fn compile(
//         source: <<Self as Compiler>::Source as Machine>::Code,
//     ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
//         let _ = source;
//         todo!("IR1 -> IR2 is not supported yet");
//     }

//     fn encode_ainput(
//         ainput: <<Self as Compiler>::Source as Machine>::AInput,
//     ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
//         Ok(ainput)
//     }

//     fn encode_rinput(
//         rinput: <<Self as Compiler>::Source as Machine>::RInput,
//     ) -> Result<<<Self as Compiler>::Target as Machine>::RInput, String> {
//         let _: () = rinput;
//         Ok(())
//     }

//     fn decode_output(
//         output: <<Self as Compiler>::Target as Machine>::Output,
//     ) -> Result<<<Self as Compiler>::Source as Machine>::Output, String> {
//         let text = output.print();
//         <crate::rec_tm_ir::Environment as TextCodec>::parse(&text)
//     }
// }

// fn compile_to_jump(program: &Ir1Program) -> Result<Ir2Program, String> {
//     let main = program
//         .functions
//         .iter()
//         .find(|func| func.name == "main")
//         .ok_or_else(|| "main() is not defined".to_string())?;
//     let body = compile_blocks(&main.blocks)?;
//     Ok(Ir2Program {
//         alphabet: program.alphabet.clone(),
//         body,
//     })
// }

// pub(crate) fn flatten_program(program: &Ir1Program) -> Result<Ir1Program, String> {
//     validate_no_recursion(program)?;
//     let main = program
//         .functions
//         .iter()
//         .find(|func| func.name == "main")
//         .ok_or_else(|| "main() is not defined".to_string())?;
//     let mut counter = 0usize;
//     let mut functions: Vec<Function> = Vec::new();
//     let blocks = expand_blocks(&main.blocks, program, &mut counter)?;
//     functions.push(Function {
//         name: "main".to_string(),
//         blocks,
//     });
//     Ok(Ir1Program {
//         alphabet: program.alphabet.clone(),
//         functions,
//     })
// }

// fn expand_blocks(
//     blocks: &[Block],
//     program: &Ir1Program,
//     counter: &mut usize,
// ) -> Result<Vec<Block>, String> {
//     let mut expanded = Vec::new();
//     for block in blocks {
//         let mut block_expanded = expand_block(block, program, counter)?;
//         expanded.append(&mut block_expanded);
//     }
//     Ok(expanded)
// }

// fn expand_block(
//     block: &Block,
//     program: &Ir1Program,
//     counter: &mut usize,
// ) -> Result<Vec<Block>, String> {
//     let mut prefix = Vec::new();
//     for (idx, stmt) in block.body.iter().enumerate() {
//         match stmt {
//             Ir1Stmt::Call { name } => {
//                 let suffix = *counter;
//                 *counter += 1;
//                 let cont_label = format!("__cont{}_{}", block.label, suffix);
//                 let callee_blocks = expand_call(name, program, counter, &cont_label, suffix)?;
//                 let mut out = Vec::new();
//                 out.push(Block {
//                     label: block.label.clone(),
//                     body: prefix,
//                 });
//                 let mut expanded_callee = expand_blocks(&callee_blocks, program, counter)?;
//                 out.append(&mut expanded_callee);
//                 let cont_block = Block {
//                     label: cont_label,
//                     body: block.body[idx + 1..].to_vec(),
//                 };
//                 let mut expanded_cont = expand_block(&cont_block, program, counter)?;
//                 out.append(&mut expanded_cont);
//                 return Ok(out);
//             }
//             _ => prefix.push(stmt.clone()),
//         }
//     }
//     Ok(vec![Block {
//         label: block.label.clone(),
//         body: prefix,
//     }])
// }

// fn expand_call(
//     name: &str,
//     program: &Ir1Program,
//     counter: &mut usize,
//     return_label: &str,
//     suffix: usize,
// ) -> Result<Vec<Block>, String> {
//     let callee = program
//         .functions
//         .iter()
//         .find(|func| func.name == name)
//         .ok_or_else(|| format!("Undefined function '{}'", name))?;
//     let var_map = build_var_map(callee, suffix);
//     let label_map = build_label_map(&callee.blocks, suffix);
//     let renamed = rename_blocks(&callee.blocks, &var_map, &label_map);
//     let replaced = replace_returns(&renamed, return_label);
//     let expanded = expand_blocks(&replaced, program, counter)?;
//     Ok(expanded)
// }

// fn build_var_map(func: &Function, suffix: usize) -> HashMap<String, String> {
//     let mut vars = HashSet::new();
//     collect_vars(&func.blocks, &mut vars);
//     vars.into_iter()
//         .map(|var| (var.clone(), format!("__flat{}_{}", suffix, var)))
//         .collect()
// }

// fn build_label_map(blocks: &[Block], suffix: usize) -> HashMap<String, String> {
//     let mut labels = HashSet::new();
//     collect_labels(blocks, &mut labels);
//     labels
//         .into_iter()
//         .map(|label| {
//             let renamed = format!("__flat{}_{}", suffix, label);
//             (label, renamed)
//         })
//         .collect()
// }

// fn rename_blocks(
//     blocks: &[Block],
//     var_map: &HashMap<String, String>,
//     label_map: &HashMap<String, String>,
// ) -> Vec<Block> {
//     blocks
//         .iter()
//         .map(|block| Block {
//             label: rename_label(&block.label, label_map),
//             body: block
//                 .body
//                 .iter()
//                 .map(|stmt| rename_stmt(stmt, var_map, label_map))
//                 .collect(),
//         })
//         .collect()
// }

// fn rename_var(var: &str, var_map: &HashMap<String, String>) -> String {
//     var_map.get(var).cloned().unwrap_or_else(|| var.to_string())
// }

// fn rename_label(label: &str, label_map: &HashMap<String, String>) -> String {
//     label_map
//         .get(label)
//         .cloned()
//         .unwrap_or_else(|| label.to_string())
// }

// fn rename_stmt(
//     stmt: &Ir1Stmt,
//     var_map: &HashMap<String, String>,
//     label_map: &HashMap<String, String>,
// ) -> Ir1Stmt {
//     match stmt {
//         Ir1Stmt::Lt => Ir1Stmt::Lt,
//         Ir1Stmt::Rt => Ir1Stmt::Rt,
//         Ir1Stmt::Read(var) => Ir1Stmt::Read(rename_var(var, var_map)),
//         Ir1Stmt::Stor(var) => Ir1Stmt::Stor(rename_var(var, var_map)),
//         Ir1Stmt::StorConst(value) => Ir1Stmt::StorConst(value.clone()),
//         Ir1Stmt::Assign(dst, src) => {
//             Ir1Stmt::Assign(rename_var(dst, var_map), rename_var(src, var_map))
//         }
//         Ir1Stmt::AssignConst(dst, value) => {
//             Ir1Stmt::AssignConst(rename_var(dst, var_map), value.clone())
//         }
//         Ir1Stmt::Jump { label } => Ir1Stmt::Jump {
//             label: rename_label(label, label_map),
//         },
//         Ir1Stmt::JumpIf { var, value, label } => Ir1Stmt::JumpIf {
//             var: rename_var(var, var_map),
//             value: value.clone(),
//             label: rename_label(label, label_map),
//         },
//         Ir1Stmt::JumpIfHead { value, label } => Ir1Stmt::JumpIfHead {
//             value: value.clone(),
//             label: rename_label(label, label_map),
//         },
//         Ir1Stmt::Return => Ir1Stmt::Return,
//         Ir1Stmt::Call { name } => Ir1Stmt::Call { name: name.clone() },
//     }
// }

// fn replace_returns(blocks: &[Block], label: &str) -> Vec<Block> {
//     blocks
//         .iter()
//         .map(|block| Block {
//             label: block.label.clone(),
//             body: block
//                 .body
//                 .iter()
//                 .map(|stmt| match stmt {
//                     Ir1Stmt::Return => Ir1Stmt::Jump {
//                         label: label.to_string(),
//                     },
//                     _ => stmt.clone(),
//                 })
//                 .collect(),
//         })
//         .collect()
// }

// fn collect_vars(blocks: &[Block], set: &mut HashSet<String>) {
//     for block in blocks {
//         for stmt in &block.body {
//             match stmt {
//                 Ir1Stmt::Read(var) | Ir1Stmt::Stor(var) => {
//                     set.insert(var.clone());
//                 }
//                 Ir1Stmt::Assign(dst, src) => {
//                     set.insert(dst.clone());
//                     set.insert(src.clone());
//                 }
//                 Ir1Stmt::AssignConst(dst, _) => {
//                     set.insert(dst.clone());
//                 }
//                 Ir1Stmt::JumpIf { var, .. } => {
//                     set.insert(var.clone());
//                 }
//                 Ir1Stmt::Call { .. } => {}
//                 Ir1Stmt::Lt | Ir1Stmt::Rt => {}
//                 Ir1Stmt::StorConst(_) => {}
//                 Ir1Stmt::Jump { .. } => {}
//                 Ir1Stmt::JumpIfHead { .. } => {}
//                 Ir1Stmt::Return => {}
//             }
//         }
//     }
// }

// fn collect_labels(blocks: &[Block], set: &mut HashSet<String>) {
//     for block in blocks {
//         set.insert(block.label.clone());
//     }
// }

// fn compile_blocks(blocks: &[Block]) -> Result<Vec<Ir2Stmt>, String> {
//     let mut labels = HashMap::new();
//     let mut flat = Vec::new();
//     for (idx, block) in blocks.iter().enumerate() {
//         if labels.contains_key(&block.label) {
//             return Err(format!("label '{}' is duplicated", block.label));
//         }
//         labels.insert(block.label.clone(), flat.len());
//         let next_label = blocks.get(idx + 1).map(|next| next.label.clone());
//         for stmt in &block.body {
//             match stmt {
//                 Ir1Stmt::Break => {
//                     if let Some(label) = next_label.clone() {
//                         flat.push(Ir1Stmt::Jump { label });
//                     } else {
//                         flat.push(Ir1Stmt::Return);
//                     }
//                 }
//                 Ir1Stmt::Continue => {
//                     flat.push(Ir1Stmt::Jump {
//                         label: block.label.clone(),
//                     });
//                 }
//                 _ => flat.push(stmt.clone()),
//             }
//         }
//     }
//     compile_flat_block(&flat, &labels)
// }

// fn compile_flat_block(
//     stmts: &[Ir1Stmt],
//     labels: &HashMap<String, usize>,
// ) -> Result<Vec<Ir2Stmt>, String> {
//     let mut instrs = Vec::new();
//     let mut return_fixups = Vec::new();
//     for stmt in stmts {
//         match stmt {
//             Ir1Stmt::Lt => instrs.push(Ir2Stmt::Lt),
//             Ir1Stmt::Rt => instrs.push(Ir2Stmt::Rt),
//             Ir1Stmt::Read(var) => instrs.push(Ir2Stmt::Read(var.clone())),
//             Ir1Stmt::Stor(var) => instrs.push(Ir2Stmt::Stor(var.clone())),
//             Ir1Stmt::StorConst(value) => instrs.push(Ir2Stmt::StorConst(value.clone())),
//             Ir1Stmt::Assign(dst, src) => instrs.push(Ir2Stmt::Assign(dst.clone(), src.clone())),
//             Ir1Stmt::AssignConst(dst, value) => {
//                 instrs.push(Ir2Stmt::ConstAssign(dst.clone(), value.clone()));
//             }
//             Ir1Stmt::Jump { label } => {
//                 let target = labels
//                     .get(label)
//                     .copied()
//                     .ok_or_else(|| format!("jump target '{}' not found", label))?;
//                 instrs.push(Ir2Stmt::Jump(target));
//             }
//             Ir1Stmt::JumpIf { var, value, label } => {
//                 let target = labels
//                     .get(label)
//                     .copied()
//                     .ok_or_else(|| format!("jump target '{}' not found", label))?;
//                 instrs.push(Ir2Stmt::JumpIf {
//                     var: var.clone(),
//                     value: value.clone(),
//                     target,
//                 });
//             }
//             Ir1Stmt::JumpIfHead { value, label } => {
//                 let target = labels
//                     .get(label)
//                     .copied()
//                     .ok_or_else(|| format!("jump target '{}' not found", label))?;
//                 instrs.push(Ir2Stmt::JumpIfHead {
//                     value: value.clone(),
//                     target,
//                 });
//             }
//             Ir1Stmt::Return => {
//                 return_fixups.push(instrs.len());
//                 instrs.push(Ir2Stmt::Jump(0));
//             }
//             Ir1Stmt::Break | Ir1Stmt::Continue => {
//                 return Err("break/continue should have been lowered".to_string());
//             }
//             Ir1Stmt::Call { name } => {
//                 return Err(format!("call '{}' should have been flattened", name));
//             }
//         }
//     }
//     let end = instrs.len();
//     for idx in return_fixups {
//         if let Ir2Stmt::Jump(target) = &mut instrs[idx] {
//             *target = end;
//         }
//     }
//     Ok(instrs)
// }
