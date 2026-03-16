use std::collections::BTreeSet;

use crate::expr_lang::{ABinOp, AExp, BExp, Environment, ExprCode, ExprLangMachine, RelOp, Stmt};
use flow_ir::flow_ir::{
    BinOp, Block, Cond, Cont, FlowIrCode, FlowIrMachine, FlowValue, JumpIf, PlaceExpr, Program,
    Region, StaticDef, StaticEnv, Stmt as IrStmt, ValueExpr,
};
use utils::number::Number;
use utils::{Compiler, TextCodec};

#[derive(Debug, Clone, Default)]
pub struct ExprLangToFlowIrCompiler;

#[derive(Debug, Default)]
struct CompileCtx {
    regions: Vec<Region>,
    region_id: usize,
    block_id: usize,
    vreg_id: usize,
}

impl CompileCtx {
    fn fresh_region(&mut self, prefix: &str) -> String {
        let id = self.region_id;
        self.region_id += 1;
        format!("{prefix}_{id}")
    }

    fn fresh_block(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{prefix}_{id}")
    }

    fn fresh_vreg(&mut self, prefix: &str) -> String {
        let id = self.vreg_id;
        self.vreg_id += 1;
        format!("{prefix}_{id}")
    }
}

fn goto_block(label: &str) -> Cont {
    Cont::Go {
        ifs: vec![],
        target: ValueExpr::CodeLabel(label.to_string()),
    }
}

fn enter_region(label: &str) -> Cont {
    Cont::Enter {
        ifs: vec![],
        target: ValueExpr::CodeLabel(label.to_string()),
    }
}

fn as_static_place(var: &str) -> PlaceExpr {
    PlaceExpr::Label(var.to_string())
}

fn move_entry_first(mut blocks: Vec<Block>, entry: &str) -> Result<Vec<Block>, String> {
    let pos = blocks
        .iter()
        .position(|b| b.label == entry)
        .ok_or_else(|| format!("entry block not found: :{entry}"))?;
    blocks.swap(0, pos);
    Ok(blocks)
}

fn compile_stmt(stmt: &Stmt, cont_region: &str, ctx: &mut CompileCtx) -> Result<String, String> {
    match stmt {
        Stmt::Nop => {
            let region_label = ctx.fresh_region("stmt_nop");
            let entry = ctx.fresh_block("entry");
            ctx.regions.push(Region {
                label: region_label.clone(),
                blocks: vec![Block {
                    label: entry,
                    stmts: vec![IrStmt::Nop],
                    cont: enter_region(cont_region),
                }],
            });
            Ok(region_label)
        }
        Stmt::Assign { var, expr } => {
            let region_label = ctx.fresh_region("stmt_assign");
            let dst = ctx.fresh_vreg("assign");
            let end = ctx.fresh_block("assign_end");
            let mut blocks = vec![Block {
                label: end.clone(),
                stmts: vec![IrStmt::Store {
                    place: as_static_place(var.as_str()),
                    src: ValueExpr::VReg(dst.clone()),
                }],
                cont: enter_region(cont_region),
            }];
            let entry = compile_aexp_blocks(expr, &dst, &end, &mut blocks, ctx)?;
            let blocks = move_entry_first(blocks, &entry)?;
            ctx.regions.push(Region {
                label: region_label.clone(),
                blocks,
            });
            Ok(region_label)
        }
        Stmt::Seq(a, b) => {
            let b_entry = compile_stmt(b, cont_region, ctx)?;
            compile_stmt(a, &b_entry, ctx)
        }
        Stmt::If { cond, body } => {
            let body_entry = compile_stmt(body, cont_region, ctx)?;
            compile_bexp_region(cond, &body_entry, cont_region, ctx)
        }
        Stmt::While { cond, body } => {
            let head_region = ctx.fresh_region("while_head");
            let body_entry = compile_stmt(body, &head_region, ctx)?;
            let br_entry = compile_bexp_region(cond, &body_entry, cont_region, ctx)?;

            let head_block = ctx.fresh_block("head_entry");
            ctx.regions.push(Region {
                label: head_region.clone(),
                blocks: vec![Block {
                    label: head_block,
                    stmts: vec![IrStmt::Nop],
                    cont: enter_region(&br_entry),
                }],
            });
            Ok(head_region)
        }
    }
}

fn compile_bexp_region(
    exp: &BExp,
    true_region: &str,
    false_region: &str,
    ctx: &mut CompileCtx,
) -> Result<String, String> {
    let region_label = ctx.fresh_region("stmt_cond");
    let true_block = ctx.fresh_block("true");
    let false_block = ctx.fresh_block("false");
    let mut blocks = vec![
        Block {
            label: true_block.clone(),
            stmts: vec![IrStmt::Nop],
            cont: enter_region(true_region),
        },
        Block {
            label: false_block.clone(),
            stmts: vec![IrStmt::Nop],
            cont: enter_region(false_region),
        },
    ];
    let entry = compile_bexp_blocks(exp, &true_block, &false_block, &mut blocks, ctx)?;
    let blocks = move_entry_first(blocks, &entry)?;
    ctx.regions.push(Region {
        label: region_label.clone(),
        blocks,
    });
    Ok(region_label)
}

fn compile_aexp_blocks(
    exp: &AExp,
    dst: &str,
    end_block: &str,
    blocks: &mut Vec<Block>,
    ctx: &mut CompileCtx,
) -> Result<String, String> {
    match exp {
        AExp::Num(n) => {
            let label = ctx.fresh_block("a_num");
            blocks.push(Block {
                label: label.clone(),
                stmts: vec![IrStmt::Assign {
                    dst: dst.to_string(),
                    src: ValueExpr::Imm(n.clone()),
                }],
                cont: goto_block(end_block),
            });
            Ok(label)
        }
        AExp::Var(v) => {
            let label = ctx.fresh_block("a_var");
            blocks.push(Block {
                label: label.clone(),
                stmts: vec![IrStmt::Load {
                    dst: dst.to_string(),
                    place: as_static_place(v.as_str()),
                }],
                cont: goto_block(end_block),
            });
            Ok(label)
        }
        AExp::BinOp { lhs, op, rhs } => {
            let lreg = ctx.fresh_vreg("lhs");
            let rreg = ctx.fresh_vreg("rhs");
            let label = ctx.fresh_block("a_bin");
            let op = match op {
                ABinOp::Add => BinOp::Add,
                ABinOp::Sub => BinOp::Sub,
            };
            blocks.push(Block {
                label: label.clone(),
                stmts: vec![IrStmt::BinOp {
                    dst: dst.to_string(),
                    lhs: ValueExpr::VReg(lreg.clone()),
                    op,
                    rhs: ValueExpr::VReg(rreg.clone()),
                }],
                cont: goto_block(end_block),
            });
            let r_entry = compile_aexp_blocks(rhs, &rreg, &label, blocks, ctx)?;
            compile_aexp_blocks(lhs, &lreg, &r_entry, blocks, ctx)
        }
        AExp::IfThenElse {
            cond,
            then_exp,
            else_exp,
        } => {
            let then_entry = compile_aexp_blocks(then_exp, dst, end_block, blocks, ctx)?;
            let else_entry = compile_aexp_blocks(else_exp, dst, end_block, blocks, ctx)?;
            compile_bexp_blocks(cond, &then_entry, &else_entry, blocks, ctx)
        }
    }
}

fn compile_bexp_blocks(
    exp: &BExp,
    true_block: &str,
    false_block: &str,
    blocks: &mut Vec<Block>,
    ctx: &mut CompileCtx,
) -> Result<String, String> {
    match exp {
        BExp::Rel { lhs, rel, rhs } => {
            let lreg = ctx.fresh_vreg("c_lhs");
            let rreg = ctx.fresh_vreg("c_rhs");
            let cond_block = ctx.fresh_block("cond");
            let cond = match rel {
                RelOp::Lt => Cond::Lt(ValueExpr::VReg(lreg.clone()), ValueExpr::VReg(rreg.clone())),
                RelOp::Eq => Cond::Eq(ValueExpr::VReg(lreg.clone()), ValueExpr::VReg(rreg.clone())),
                RelOp::Gt => Cond::Gt(ValueExpr::VReg(lreg.clone()), ValueExpr::VReg(rreg.clone())),
            };
            blocks.push(Block {
                label: cond_block.clone(),
                stmts: vec![],
                cont: Cont::Go {
                    ifs: vec![JumpIf {
                        cond,
                        target: ValueExpr::CodeLabel(true_block.to_string()),
                    }],
                    target: ValueExpr::CodeLabel(false_block.to_string()),
                },
            });
            let r_entry = compile_aexp_blocks(rhs, &rreg, &cond_block, blocks, ctx)?;
            compile_aexp_blocks(lhs, &lreg, &r_entry, blocks, ctx)
        }
        BExp::Or(a, b) => {
            let b_entry = compile_bexp_blocks(b, true_block, false_block, blocks, ctx)?;
            compile_bexp_blocks(a, true_block, &b_entry, blocks, ctx)
        }
        BExp::Not(b) => compile_bexp_blocks(b, false_block, true_block, blocks, ctx),
    }
}

impl ExprLangToFlowIrCompiler {
    pub fn compile(code: &ExprCode) -> Result<String, String> {
        let ir = <Self as Compiler>::compile(code.clone())?;
        Ok(ir.print())
    }
}

impl Compiler for ExprLangToFlowIrCompiler {
    type Source = ExprLangMachine;
    type Target = FlowIrMachine;

    fn compile(source: ExprCode) -> Result<FlowIrCode, String> {
        let vars: Vec<String> = collect_vars(&source.0).into_iter().collect();

        let mut ctx = CompileCtx::default();
        let halt_region = ctx.fresh_region("halt");
        let halt_block = ctx.fresh_block("halt_entry");
        ctx.regions.push(Region {
            label: halt_region.clone(),
            blocks: vec![Block {
                label: halt_block,
                stmts: vec![IrStmt::Nop],
                cont: Cont::Halt { ifs: vec![] },
            }],
        });

        let entry_region = compile_stmt(&source.0, &halt_region, &mut ctx)?;

        let main_region = Region {
            label: "main".to_string(),
            blocks: vec![Block {
                label: "entry".to_string(),
                stmts: vec![IrStmt::Nop],
                cont: enter_region(&entry_region),
            }],
        };

        let statics = vars
            .iter()
            .map(|name| StaticDef {
                label: name.clone(),
                value: Number::from(0usize),
            })
            .collect();

        let mut regions = vec![main_region];
        regions.extend(ctx.regions);

        Ok(FlowIrCode(Program { statics, regions }))
    }

    fn encode_ainput(
        ainput: <Self::Source as utils::Machine>::AInput,
    ) -> Result<<Self::Target as utils::Machine>::AInput, String> {
        let mut entries = std::collections::BTreeMap::new();
        for (var, num) in ainput.vars {
            entries.insert(var.as_str().to_string(), FlowValue::Num(num));
        }
        Ok(StaticEnv { entries })
    }

    fn encode_rinput(
        rinput: <Self::Source as utils::Machine>::RInput,
    ) -> Result<<Self::Target as utils::Machine>::RInput, String> {
        let _: () = rinput;
        Ok(String::new())
    }

    fn decode_routput(
        output: <Self::Target as utils::Machine>::ROutput,
    ) -> Result<<Self::Source as utils::Machine>::ROutput, String> {
        let _: String = output;
        Ok(())
    }

    fn decode_foutput(
        output: <Self::Target as utils::Machine>::FOutput,
    ) -> Result<<Self::Source as utils::Machine>::FOutput, String> {
        let mut env = Environment::default();
        for (name, value) in output.entries {
            let num = match value {
                FlowValue::Num(n) => n,
                other => {
                    return Err(format!(
                        "expr decode expects numeric static value for {name}, got {}",
                        other.print()
                    ));
                }
            };
            env.set(
                utils::identifier::Identifier::new(&name).map_err(|e| e.to_string())?,
                num,
            );
        }
        Ok(env)
    }
}

fn collect_vars(stmt: &Stmt) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    collect_stmt_vars(stmt, &mut vars);
    vars
}

fn collect_stmt_vars(stmt: &Stmt, vars: &mut BTreeSet<String>) {
    match stmt {
        Stmt::Nop => {}
        Stmt::Assign { var, expr } => {
            vars.insert(var.as_str().to_string());
            collect_aexp_vars(expr, vars);
        }
        Stmt::Seq(a, b) => {
            collect_stmt_vars(a, vars);
            collect_stmt_vars(b, vars);
        }
        Stmt::If { cond, body } => {
            collect_bexp_vars(cond, vars);
            collect_stmt_vars(body, vars);
        }
        Stmt::While { cond, body } => {
            collect_bexp_vars(cond, vars);
            collect_stmt_vars(body, vars);
        }
    }
}

fn collect_aexp_vars(exp: &AExp, vars: &mut BTreeSet<String>) {
    match exp {
        AExp::Var(v) => {
            vars.insert(v.as_str().to_string());
        }
        AExp::Num(_) => {}
        AExp::BinOp { lhs, rhs, .. } => {
            collect_aexp_vars(lhs, vars);
            collect_aexp_vars(rhs, vars);
        }
        AExp::IfThenElse {
            cond,
            then_exp,
            else_exp,
        } => {
            collect_bexp_vars(cond, vars);
            collect_aexp_vars(then_exp, vars);
            collect_aexp_vars(else_exp, vars);
        }
    }
}

fn collect_bexp_vars(exp: &BExp, vars: &mut BTreeSet<String>) {
    match exp {
        BExp::Rel { lhs, rhs, .. } => {
            collect_aexp_vars(lhs, vars);
            collect_aexp_vars(rhs, vars);
        }
        BExp::Or(a, b) => {
            collect_bexp_vars(a, vars);
            collect_bexp_vars(b, vars);
        }
        BExp::Not(b) => collect_bexp_vars(b, vars),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_emits_parseable_flow_ir() {
        let code = ExprCode::parse("x := 0 ; while x < 3 [ x := x + 1 ]").unwrap();
        let ir = <ExprLangToFlowIrCompiler as Compiler>::compile(code).unwrap();
        let printed = ir.print();
        let reparsed = FlowIrCode::parse(&printed).unwrap();
        assert_eq!(ir, reparsed);
    }
}
