use std::collections::HashMap;

use super::{ABinOp, AExp, Atom, GlobalEnv, ProcCode, ProcDef, ProcLangMachine, RelOp, Stmt};
use flow_ir::flow_ir::{
    BinOp as IrBinOp, Block, Cond as IrCond, Cont, FlowIrCode, FlowIrMachine, FlowValue, JumpIf,
    PlaceExpr, Program as IrProgram, Region, StaticDef, StaticEnv, Stmt as IrStmt, ValueExpr,
};
use utils::number::Number;
use utils::{Compiler, Machine};

#[derive(Debug, Clone, Default)]
pub struct ProcToFlowIrCompiler;

#[derive(Debug, Clone, Copy)]
enum VarLoc {
    Global,
    Param(usize),
    Local(usize),
}

#[derive(Debug, Clone)]
struct ProcScope<'a> {
    locals_len: usize,
    params_len: usize,
    vars: HashMap<&'a str, VarLoc>,
    proc_entry_region: &'a HashMap<String, String>,
}

#[derive(Debug, Default)]
struct CompileCtx {
    regions: Vec<Region>,
    region_id: usize,
    vreg_id: usize,
}

impl CompileCtx {
    fn fresh_region(&mut self, prefix: &str) -> String {
        let id = self.region_id;
        self.region_id += 1;
        format!("{prefix}_{id}")
    }

    fn fresh_vreg(&mut self, prefix: &str) -> String {
        let id = self.vreg_id;
        self.vreg_id += 1;
        format!("{prefix}_{id}")
    }
}

fn ensure_with_return(stmt: Stmt) -> Stmt {
    match stmt {
        s if s.ends_with_return() => s,
        s => Stmt::Seq(Box::new(s), Box::new(Stmt::Return { vars: vec![] })),
    }
}

fn enter_region(label: &str) -> Cont {
    Cont::Enter {
        ifs: vec![],
        target: ValueExpr::CodeLabel(label.to_string()),
    }
}

fn stack_index_from_top_stmts(offset_from_top: usize, dst_idx: &str, ctx: &mut CompileCtx) -> Vec<IrStmt> {
    let len = ctx.fresh_vreg("len");
    vec![
        IrStmt::LGet {
            dst: len.clone(),
        },
        IrStmt::BinOp {
            dst: dst_idx.to_string(),
            lhs: ValueExpr::VReg(len),
            op: IrBinOp::Sub,
            rhs: ValueExpr::Imm(Number::from(offset_from_top)),
        },
    ]
}

fn emit_load_var(
    var: &str,
    dst: &str,
    scope: &ProcScope<'_>,
    ctx: &mut CompileCtx,
    stmts: &mut Vec<IrStmt>,
) -> Result<(), String> {
    let loc = scope
        .vars
        .get(var)
        .copied()
        .ok_or_else(|| format!("Unknown variable in compiler: {var}"))?;
    match loc {
        VarLoc::Global => stmts.push(IrStmt::Load {
            dst: dst.to_string(),
            place: PlaceExpr::Label(var.to_string()),
        }),
        VarLoc::Local(i) => {
            let idx = ctx.fresh_vreg("idx_local");
            stmts.extend(stack_index_from_top_stmts(i + 1, &idx, ctx));
            stmts.push(IrStmt::Load {
                dst: dst.to_string(),
                place: PlaceExpr::SAcc(Box::new(ValueExpr::VReg(idx))),
            });
        }
        VarLoc::Param(i) => {
            let idx = ctx.fresh_vreg("idx_param");
            let offset = scope.locals_len + 2 + i;
            stmts.extend(stack_index_from_top_stmts(offset, &idx, ctx));
            stmts.push(IrStmt::Load {
                dst: dst.to_string(),
                place: PlaceExpr::SAcc(Box::new(ValueExpr::VReg(idx))),
            });
        }
    }
    Ok(())
}

fn emit_store_var(
    var: &str,
    src: ValueExpr,
    scope: &ProcScope<'_>,
    ctx: &mut CompileCtx,
    stmts: &mut Vec<IrStmt>,
) -> Result<(), String> {
    let loc = scope
        .vars
        .get(var)
        .copied()
        .ok_or_else(|| format!("Unknown variable in compiler: {var}"))?;
    match loc {
        VarLoc::Global => stmts.push(IrStmt::Store {
            place: PlaceExpr::Label(var.to_string()),
            src,
        }),
        VarLoc::Local(i) => {
            let idx = ctx.fresh_vreg("idx_local");
            stmts.extend(stack_index_from_top_stmts(i + 1, &idx, ctx));
            stmts.push(IrStmt::Store {
                place: PlaceExpr::SAcc(Box::new(ValueExpr::VReg(idx))),
                src,
            });
        }
        VarLoc::Param(i) => {
            let idx = ctx.fresh_vreg("idx_param");
            let offset = scope.locals_len + 2 + i;
            stmts.extend(stack_index_from_top_stmts(offset, &idx, ctx));
            stmts.push(IrStmt::Store {
                place: PlaceExpr::SAcc(Box::new(ValueExpr::VReg(idx))),
                src,
            });
        }
    }
    Ok(())
}

fn emit_atom_to_vreg(
    atom: &Atom,
    dst: &str,
    scope: &ProcScope<'_>,
    ctx: &mut CompileCtx,
    stmts: &mut Vec<IrStmt>,
) -> Result<(), String> {
    match atom {
        Atom::Imm(n) => {
            stmts.push(IrStmt::Assign {
                dst: dst.to_string(),
                src: ValueExpr::Imm(n.clone()),
            });
        }
        Atom::Var(v) => emit_load_var(v, dst, scope, ctx, stmts)?,
    }
    Ok(())
}

fn emit_aexp_to_vreg(
    exp: &AExp,
    dst: &str,
    scope: &ProcScope<'_>,
    ctx: &mut CompileCtx,
    stmts: &mut Vec<IrStmt>,
) -> Result<(), String> {
    match exp {
        AExp::Atom(a) => emit_atom_to_vreg(a, dst, scope, ctx, stmts),
        AExp::Bin { lhs, op, rhs } => {
            let l = ctx.fresh_vreg("lhs");
            let r = ctx.fresh_vreg("rhs");
            emit_atom_to_vreg(lhs, &l, scope, ctx, stmts)?;
            emit_atom_to_vreg(rhs, &r, scope, ctx, stmts)?;
            let op = match op {
                ABinOp::Add => IrBinOp::Add,
                ABinOp::Sub => IrBinOp::Sub,
            };
            stmts.push(IrStmt::BinOp {
                dst: dst.to_string(),
                lhs: ValueExpr::VReg(l),
                op,
                rhs: ValueExpr::VReg(r),
            });
            Ok(())
        }
    }
}

fn compile_return_region(
    vars: &[String],
    scope: &ProcScope<'_>,
    ctx: &mut CompileCtx,
) -> Result<String, String> {
    let label = ctx.fresh_region("ret");
    let mut stmts = Vec::new();
    let mut ret_regs = Vec::new();

    for v in vars {
        let reg = ctx.fresh_vreg("retv");
        emit_load_var(v, &reg, scope, ctx, &mut stmts)?;
        ret_regs.push(reg);
    }

    for _ in 0..scope.locals_len {
        stmts.push(IrStmt::Pop {
            dst: ctx.fresh_vreg("drop_local"),
        });
    }
    let ret_addr = ctx.fresh_vreg("ret_addr");
    stmts.push(IrStmt::Pop {
        dst: ret_addr.clone(),
    });
    for _ in 0..scope.params_len {
        stmts.push(IrStmt::Pop {
            dst: ctx.fresh_vreg("drop_param"),
        });
    }

    for reg in ret_regs.iter().rev() {
        stmts.push(IrStmt::Push {
            src: ValueExpr::VReg(reg.clone()),
        });
    }

    ctx.regions.push(Region {
        label: label.clone(),
        blocks: vec![Block {
            label: "entry".to_string(),
            stmts,
            cont: Cont::Enter {
                ifs: vec![],
                target: ValueExpr::VReg(ret_addr),
            },
        }],
    });
    Ok(label)
}

fn compile_stmt_to_region(
    stmt: &Stmt,
    cont_region: &str,
    scope: &ProcScope<'_>,
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
        Stmt::Assign { var, expr } => {
            let label = ctx.fresh_region("assign");
            let dst = ctx.fresh_vreg("assign");
            let mut stmts = Vec::new();
            emit_aexp_to_vreg(expr, &dst, scope, ctx, &mut stmts)?;
            emit_store_var(var, ValueExpr::VReg(dst), scope, ctx, &mut stmts)?;
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
        Stmt::Seq(a, b) => {
            let b_region = compile_stmt_to_region(b, cont_region, scope, ctx)?;
            compile_stmt_to_region(a, &b_region, scope, ctx)
        }
        Stmt::If { cond, body } => {
            let body_region = compile_stmt_to_region(body, cont_region, scope, ctx)?;
            let label = ctx.fresh_region("if");
            let lhs = ctx.fresh_vreg("if_lhs");
            let rhs = ctx.fresh_vreg("if_rhs");
            let mut stmts = Vec::new();
            emit_atom_to_vreg(&cond.lhs, &lhs, scope, ctx, &mut stmts)?;
            emit_atom_to_vreg(&cond.rhs, &rhs, scope, ctx, &mut stmts)?;
            let cond = match cond.op {
                RelOp::Lt => IrCond::Lt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
                RelOp::Eq => IrCond::Eq(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
                RelOp::Gt => IrCond::Gt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
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
        Stmt::While { cond, body } => {
            let test_region = ctx.fresh_region("while_test");
            let body_region = compile_stmt_to_region(body, &test_region, scope, ctx)?;
            let lhs = ctx.fresh_vreg("while_lhs");
            let rhs = ctx.fresh_vreg("while_rhs");
            let mut stmts = Vec::new();
            emit_atom_to_vreg(&cond.lhs, &lhs, scope, ctx, &mut stmts)?;
            emit_atom_to_vreg(&cond.rhs, &rhs, scope, ctx, &mut stmts)?;
            let cond = match cond.op {
                RelOp::Lt => IrCond::Lt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
                RelOp::Eq => IrCond::Eq(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
                RelOp::Gt => IrCond::Gt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
            };
            ctx.regions.push(Region {
                label: test_region.clone(),
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
            Ok(test_region)
        }
        Stmt::Call { name, args, rets } => {
            let ret_region = ctx.fresh_region("call_ret");
            let ret_label = ValueExpr::CodeLabel(ret_region.clone());
            let mut call_stmts = Vec::new();

            for arg in args.iter().rev() {
                let reg = ctx.fresh_vreg("arg");
                emit_load_var(arg, &reg, scope, ctx, &mut call_stmts)?;
                call_stmts.push(IrStmt::Push {
                    src: ValueExpr::VReg(reg),
                });
            }
            call_stmts.push(IrStmt::Push { src: ret_label });

            let target = scope
                .proc_entry_region
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Unknown procedure in compiler: {name}"))?;

            let call_region = ctx.fresh_region("call");
            ctx.regions.push(Region {
                label: call_region.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: call_stmts,
                    cont: enter_region(&target),
                }],
            });

            let mut ret_stmts = Vec::new();
            for ret in rets {
                let reg = ctx.fresh_vreg("ret");
                ret_stmts.push(IrStmt::Pop { dst: reg.clone() });
                emit_store_var(ret, ValueExpr::VReg(reg), scope, ctx, &mut ret_stmts)?;
            }
            ctx.regions.push(Region {
                label: ret_region.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: ret_stmts,
                    cont: enter_region(cont_region),
                }],
            });
            Ok(call_region)
        }
        Stmt::Return { vars } => compile_return_region(vars, scope, ctx),
    }
}

fn build_scope<'a>(
    program: &'a ProcCode,
    proc: &'a ProcDef,
    proc_entry_region: &'a HashMap<String, String>,
) -> Result<ProcScope<'a>, String> {
    let mut vars = HashMap::new();
    for g in &program.0.statics {
        vars.insert(g.as_str(), VarLoc::Global);
    }
    for (i, p) in proc.params.iter().enumerate() {
        vars.insert(p.as_str(), VarLoc::Param(i));
    }
    for (i, l) in proc.locals.iter().enumerate() {
        vars.insert(l.as_str(), VarLoc::Local(i));
    }
    Ok(ProcScope {
        locals_len: proc.locals.len(),
        params_len: proc.params.len(),
        vars,
        proc_entry_region,
    })
}

impl Compiler for ProcToFlowIrCompiler {
    type Source = ProcLangMachine;
    type Target = FlowIrMachine;

    fn compile(source: ProcCode) -> Result<FlowIrCode, String> {
        let mut ctx = CompileCtx::default();
        let halt_region = ctx.fresh_region("halt");
        ctx.regions.push(Region {
            label: halt_region.clone(),
            blocks: vec![Block {
                label: "entry".to_string(),
                stmts: vec![IrStmt::Nop],
                cont: Cont::Halt { ifs: vec![] },
            }],
        });

        let mut entry_regions = HashMap::new();
        for proc in &source.0.procs {
            entry_regions.insert(proc.name.clone(), ctx.fresh_region(&format!("proc_{}", proc.name)));
        }

        for proc in &source.0.procs {
            let scope = build_scope(&source, proc, &entry_regions)?;
            let body = ensure_with_return(proc.body.clone());
            let body_region = compile_stmt_to_region(&body, &halt_region, &scope, &mut ctx)?;

            let mut stmts = Vec::new();
            for _ in &proc.locals {
                stmts.push(IrStmt::Push {
                    src: ValueExpr::Imm(Number::default()),
                });
            }
            let entry = entry_regions.get(&proc.name).unwrap().clone();
            ctx.regions.push(Region {
                label: entry,
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts,
                    cont: enter_region(&body_region),
                }],
            });
        }

        let main_entry = entry_regions
            .get("main")
            .cloned()
            .ok_or_else(|| "Procedure 'main' is required".to_string())?;
        let mut main_stmts = vec![IrStmt::Push {
            src: ValueExpr::CodeLabel(halt_region.clone()),
        }];
        ctx.regions.push(Region {
            label: "main".to_string(),
            blocks: vec![Block {
                label: "entry".to_string(),
                stmts: {
                    let s = &mut main_stmts;
                    s.push(IrStmt::Nop);
                    s.clone()
                },
                cont: enter_region(&main_entry),
            }],
        });

        let statics = source
            .0
            .statics
            .iter()
            .cloned()
            .map(|label| StaticDef {
                label,
                value: Number::default(),
            })
            .collect();

        Ok(FlowIrCode(IrProgram {
            statics,
            regions: ctx.regions,
        }))
    }

    fn encode_ainput(
        ainput: <Self::Source as Machine>::AInput,
    ) -> Result<<Self::Target as Machine>::AInput, String> {
        Ok(StaticEnv {
            entries: ainput
                .vars
                .into_iter()
                .map(|(k, v)| (k, FlowValue::Num(v)))
                .collect(),
        })
    }

    fn encode_rinput(
        rinput: <Self::Source as Machine>::RInput,
    ) -> Result<<Self::Target as Machine>::RInput, String> {
        Ok(rinput)
    }

    fn decode_routput(
        output: <Self::Target as Machine>::ROutput,
    ) -> Result<<Self::Source as Machine>::ROutput, String> {
        Ok(output)
    }

    fn decode_foutput(
        output: <Self::Target as Machine>::FOutput,
    ) -> Result<<Self::Source as Machine>::FOutput, String> {
        let mut vars = std::collections::BTreeMap::new();
        for (k, v) in output.entries {
            match v {
                FlowValue::Num(n) => {
                    vars.insert(k, n);
                }
                other => return Err(format!("expected numeric global for {k}, got {}", other.print())),
            }
        }
        Ok(GlobalEnv { vars })
    }
}
