use std::collections::{BTreeSet, HashMap};

use crate::internal_ctrl::{
    ABinOp, AExp, Atom, BExp, Environment, InternalCtrlCode, InternalCtrlMachine, RelOp, Stmt,
};
use flow_ir::flow_ir::{
    BinOp, Block, Cond, Cont, FlowIrCode, FlowIrMachine, FlowValue, JumpIf, PlaceExpr, Program,
    Region, StaticDef, StaticEnv, Stmt as IrStmt, ValueExpr,
};
use utils::number::Number;
use utils::{Compiler, Machine, TextCodec};

#[derive(Debug, Clone, Default)]
pub struct InternalCtrlToFlowIrCompiler;

#[derive(Debug, Clone)]
struct LoopTarget {
    head_region: String,
    exit_region: String,
}

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

fn enter_region(label: &str) -> Cont {
    Cont::Enter {
        ifs: vec![],
        target: ValueExpr::CodeLabel(label.to_string()),
    }
}

fn as_static_place(var: &str) -> PlaceExpr {
    PlaceExpr::Label(var.to_string())
}

fn emit_atom(atom: &Atom, dst: &str, stmts: &mut Vec<IrStmt>) {
    match atom {
        Atom::Var(v) => stmts.push(IrStmt::Load {
            dst: dst.to_string(),
            place: as_static_place(v.as_str()),
        }),
        Atom::Imm(n) => stmts.push(IrStmt::Assign {
            dst: dst.to_string(),
            src: ValueExpr::Imm(n.clone()),
        }),
    }
}

fn emit_aexp(exp: &AExp, dst: &str, ctx: &mut CompileCtx, stmts: &mut Vec<IrStmt>) {
    match exp {
        AExp::Atom(atom) => emit_atom(atom, dst, stmts),
        AExp::Bin { lhs, op, rhs } => {
            let lhs_reg = ctx.fresh_vreg("lhs");
            let rhs_reg = ctx.fresh_vreg("rhs");
            emit_atom(lhs, &lhs_reg, stmts);
            emit_atom(rhs, &rhs_reg, stmts);
            let op = match op {
                ABinOp::Add => BinOp::Add,
                ABinOp::Sub => BinOp::Sub,
            };
            stmts.push(IrStmt::BinOp {
                dst: dst.to_string(),
                lhs: ValueExpr::VReg(lhs_reg),
                op,
                rhs: ValueExpr::VReg(rhs_reg),
            });
        }
    }
}

fn compile_stmt(
    stmt: &Stmt,
    cont_region: &str,
    loops: &HashMap<String, LoopTarget>,
    ctx: &mut CompileCtx,
) -> Result<String, String> {
    match stmt {
        Stmt::Nop => {
            let label = ctx.fresh_region("nop");
            ctx.regions.push(Region {
                label: label.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: vec![IrStmt::Nop],
                    cont: enter_region(cont_region),
                }],
            });
            Ok(label)
        }
        Stmt::Seq(lhs, rhs) => {
            let rhs_region = compile_stmt(rhs, cont_region, loops, ctx)?;
            compile_stmt(lhs, &rhs_region, loops, ctx)
        }
        Stmt::Assign { var, expr } => {
            let label = ctx.fresh_region("assign");
            let dst = ctx.fresh_vreg("assign");
            let mut stmts = Vec::new();
            emit_aexp(expr, &dst, ctx, &mut stmts);
            stmts.push(IrStmt::Store {
                place: as_static_place(var.as_str()),
                src: ValueExpr::VReg(dst),
            });
            ctx.regions.push(Region {
                label: label.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts,
                    cont: enter_region(cont_region),
                }],
            });
            Ok(label)
        }
        Stmt::If { cond, body } => {
            let body_region = compile_stmt(body, cont_region, loops, ctx)?;
            let label = ctx.fresh_region("if");
            let lhs = ctx.fresh_vreg("if_lhs");
            let rhs = ctx.fresh_vreg("if_rhs");
            let mut stmts = Vec::new();
            emit_atom(&cond.lhs, &lhs, &mut stmts);
            emit_atom(&cond.rhs, &rhs, &mut stmts);
            let cond = match cond.rel {
                RelOp::Lt => Cond::Lt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
                RelOp::Eq => Cond::Eq(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
                RelOp::Gt => Cond::Gt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
            };
            ctx.regions.push(Region {
                label: label.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts,
                    cont: Cont::Enter {
                        ifs: vec![JumpIf {
                            cond,
                            target: ValueExpr::CodeLabel(body_region),
                        }],
                        target: ValueExpr::CodeLabel(cont_region.to_string()),
                    },
                }],
            });
            Ok(label)
        }
        Stmt::Break { label, value } => {
            let Some(target) = loops.get(label.as_str()) else {
                return Err(format!("unknown loop label: {}", label.as_str()));
            };
            let region = ctx.fresh_region("break");
            let reg = ctx.fresh_vreg("break");
            let mut stmts = Vec::new();
            stmts.push(IrStmt::Load {
                dst: reg.clone(),
                place: as_static_place(value.as_str()),
            });
            stmts.push(IrStmt::Push {
                src: ValueExpr::VReg(reg),
            });
            ctx.regions.push(Region {
                label: region.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts,
                    cont: enter_region(&target.exit_region),
                }],
            });
            Ok(region)
        }
        Stmt::Continue { label } => {
            let Some(target) = loops.get(label.as_str()) else {
                return Err(format!("unknown loop label: {}", label.as_str()));
            };
            let region = ctx.fresh_region("continue");
            ctx.regions.push(Region {
                label: region.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: vec![IrStmt::Nop],
                    cont: enter_region(&target.head_region),
                }],
            });
            Ok(region)
        }
        Stmt::Loop { label, body, out } => {
            let head_region = ctx.fresh_region("loop_head");
            let exit_region = ctx.fresh_region("loop_exit");
            let mut nested = loops.clone();
            nested.insert(
                label.as_str().to_string(),
                LoopTarget {
                    head_region: head_region.clone(),
                    exit_region: exit_region.clone(),
                },
            );
            let body_region = compile_stmt(body, &head_region, &nested, ctx)?;
            ctx.regions.push(Region {
                label: head_region.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: vec![IrStmt::Nop],
                    cont: enter_region(&body_region),
                }],
            });
            let value_reg = ctx.fresh_vreg("loop_break");
            ctx.regions.push(Region {
                label: exit_region,
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: vec![
                        IrStmt::Pop {
                            dst: value_reg.clone(),
                        },
                        IrStmt::Store {
                            place: as_static_place(out.as_str()),
                            src: ValueExpr::VReg(value_reg),
                        },
                    ],
                    cont: enter_region(cont_region),
                }],
            });
            Ok(head_region)
        }
    }
}

impl InternalCtrlToFlowIrCompiler {
    pub fn compile(code: &InternalCtrlCode) -> Result<String, String> {
        let ir = <Self as Compiler>::compile(code.clone())?;
        Ok(ir.print())
    }
}

impl Compiler for InternalCtrlToFlowIrCompiler {
    type Source = InternalCtrlMachine;
    type Target = FlowIrMachine;

    fn compile(source: InternalCtrlCode) -> Result<FlowIrCode, String> {
        let mut statics: BTreeSet<String> = source
            .statics
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        collect_stmt_vars(&source.body, &mut statics);

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

        let entry_region = compile_stmt(&source.body, &halt_region, &HashMap::new(), &mut ctx)?;
        let main_region = Region {
            label: "main".to_string(),
            blocks: vec![Block {
                label: "entry".to_string(),
                stmts: vec![IrStmt::Nop],
                cont: enter_region(&entry_region),
            }],
        };

        let mut regions = vec![main_region];
        regions.extend(ctx.regions);
        let statics = statics
            .into_iter()
            .map(|label| StaticDef {
                label,
                value: Number::from(0usize),
            })
            .collect();
        Ok(FlowIrCode(Program { statics, regions }))
    }

    fn encode_ainput(
        ainput: <Self::Source as Machine>::AInput,
    ) -> Result<<Self::Target as Machine>::AInput, String> {
        let entries = ainput
            .vars
            .into_iter()
            .map(|(name, value)| (name.as_str().to_string(), FlowValue::Num(value)))
            .collect();
        Ok(StaticEnv { entries })
    }

    fn encode_rinput(
        _rinput: <Self::Source as Machine>::RInput,
    ) -> Result<<Self::Target as Machine>::RInput, String> {
        Ok(String::new())
    }

    fn decode_routput(
        _output: <Self::Target as Machine>::ROutput,
    ) -> Result<<Self::Source as Machine>::ROutput, String> {
        Ok(String::new())
    }

    fn decode_foutput(
        output: <Self::Target as Machine>::FOutput,
    ) -> Result<<Self::Source as Machine>::FOutput, String> {
        let mut env = Environment::default();
        for (name, value) in output.entries {
            let num = match value {
                FlowValue::Num(num) => num,
                other => {
                    return Err(format!(
                        "internal_ctrl decode expects numeric static value for {name}, got {}",
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

fn collect_stmt_vars(stmt: &Stmt, vars: &mut BTreeSet<String>) {
    match stmt {
        Stmt::Nop => {}
        Stmt::Seq(lhs, rhs) => {
            collect_stmt_vars(lhs, vars);
            collect_stmt_vars(rhs, vars);
        }
        Stmt::Assign { var, expr } => {
            vars.insert(var.as_str().to_string());
            collect_aexp_vars(expr, vars);
        }
        Stmt::If { cond, body } => {
            collect_bexp_vars(cond, vars);
            collect_stmt_vars(body, vars);
        }
        Stmt::Break { value, .. } => {
            vars.insert(value.as_str().to_string());
        }
        Stmt::Continue { .. } => {}
        Stmt::Loop { body, out, .. } => {
            vars.insert(out.as_str().to_string());
            collect_stmt_vars(body, vars);
        }
    }
}

fn collect_aexp_vars(exp: &AExp, vars: &mut BTreeSet<String>) {
    match exp {
        AExp::Atom(Atom::Var(v)) => {
            vars.insert(v.as_str().to_string());
        }
        AExp::Atom(Atom::Imm(_)) => {}
        AExp::Bin { lhs, rhs, .. } => {
            if let Atom::Var(v) = lhs {
                vars.insert(v.as_str().to_string());
            }
            if let Atom::Var(v) = rhs {
                vars.insert(v.as_str().to_string());
            }
        }
    }
}

fn collect_bexp_vars(exp: &BExp, vars: &mut BTreeSet<String>) {
    if let Atom::Var(v) = &exp.lhs {
        vars.insert(v.as_str().to_string());
    }
    if let Atom::Var(v) = &exp.rhs {
        vars.insert(v.as_str().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_emits_parseable_flow_ir() {
        let code = InternalCtrlCode::parse(
            "static x, y; loop :outer ( if x < 3 { x := x + 1; continue :outer }; break :outer x ) -> y",
        )
        .unwrap();
        let ir = <InternalCtrlToFlowIrCompiler as Compiler>::compile(code).unwrap();
        let printed = ir.print();
        let reparsed = FlowIrCode::parse(&printed).unwrap();
        assert_eq!(ir, reparsed);
    }
}
