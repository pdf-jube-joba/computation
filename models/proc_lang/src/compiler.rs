use std::collections::HashMap;

use flow_ir::flow_ir::{
    BinOp as IrBinOp, Block, Cond as IrCond, Cont, FlowIrCode, FlowIrMachine, FlowValue, JumpIf,
    PlaceExpr, Program as IrProgram, Region, StaticDef, StaticEnv, Stmt as IrStmt, ValueExpr,
};
use utils::number::Number;
use utils::{Compiler, Machine};

use crate::{ABinOp, AExp, Atom, BExp, GlobalEnv, ProcCode, ProcDef, ProcLangMachine, RelOp, Stmt};

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

fn compile_cond_region(
    cond: &BExp,
    true_region: &str,
    false_region: &str,
    scope: &ProcScope<'_>,
    ctx: &mut CompileCtx,
) -> Result<String, String> {
    let region = ctx.fresh_region("cond");
    let true_block = ctx.fresh_block("true");
    let false_block = ctx.fresh_block("false");
    let mut stmts = Vec::new();

    let lhs = ctx.fresh_vreg("cond_lhs");
    let rhs = ctx.fresh_vreg("cond_rhs");
    emit_atom_to_vreg(&cond.lhs, &lhs, scope, ctx, &mut stmts)?;
    emit_atom_to_vreg(&cond.rhs, &rhs, scope, ctx, &mut stmts)?;

    let cond = match cond.op {
        RelOp::Lt => IrCond::Lt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
        RelOp::Eq => IrCond::Eq(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
        RelOp::Gt => IrCond::Gt(ValueExpr::VReg(lhs), ValueExpr::VReg(rhs)),
    };

    ctx.regions.push(Region {
        label: region.clone(),
        blocks: vec![
            Block {
                label: "entry".to_string(),
                stmts,
                cont: Cont::Go {
                    ifs: vec![JumpIf {
                        cond,
                        target: ValueExpr::CodeLabel(true_block.clone()),
                    }],
                    target: ValueExpr::CodeLabel(false_block.clone()),
                },
            },
            Block {
                label: true_block,
                stmts: vec![IrStmt::Nop],
                cont: enter_region(true_region),
            },
            Block {
                label: false_block,
                stmts: vec![IrStmt::Nop],
                cont: enter_region(false_region),
            },
        ],
    });
    Ok(region)
}

fn compile_stmt(
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
            let mut stmts = Vec::new();
            let v = ctx.fresh_vreg("assign");
            emit_aexp_to_vreg(expr, &v, scope, ctx, &mut stmts)?;
            emit_store_var(var, ValueExpr::VReg(v), scope, ctx, &mut stmts)?;
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
            let b_entry = compile_stmt(b, cont_region, scope, ctx)?;
            compile_stmt(a, &b_entry, scope, ctx)
        }
        Stmt::If { cond, body } => {
            let body_entry = compile_stmt(body, cont_region, scope, ctx)?;
            compile_cond_region(cond, &body_entry, cont_region, scope, ctx)
        }
        Stmt::While { cond, body } => {
            let head = ctx.fresh_region("while_head");
            let body_entry = compile_stmt(body, &head, scope, ctx)?;
            let cond_entry = compile_cond_region(cond, &body_entry, cont_region, scope, ctx)?;
            ctx.regions.push(Region {
                label: head.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: vec![IrStmt::Nop],
                    cont: enter_region(&cond_entry),
                }],
            });
            Ok(head)
        }
        Stmt::Call { name, args, rets } => {
            let callee_entry = scope
                .proc_entry_region
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Unknown callee in compiler: {name}"))?;

            let ret_region = ctx.fresh_region("call_ret");
            let call_region = ctx.fresh_region("call");

            let mut call_stmts = Vec::new();
            for arg in args.iter().rev() {
                let v = ctx.fresh_vreg("arg");
                emit_load_var(arg, &v, scope, ctx, &mut call_stmts)?;
                call_stmts.push(IrStmt::Push {
                    src: ValueExpr::VReg(v),
                });
            }
            call_stmts.push(IrStmt::Push {
                src: ValueExpr::CodeLabel(ret_region.clone()),
            });

            ctx.regions.push(Region {
                label: call_region.clone(),
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: call_stmts,
                    cont: enter_region(&callee_entry),
                }],
            });

            let mut ret_stmts = Vec::new();
            for ret in rets {
                let v = ctx.fresh_vreg("ret");
                ret_stmts.push(IrStmt::Pop { dst: v.clone() });
                emit_store_var(ret, ValueExpr::VReg(v), scope, ctx, &mut ret_stmts)?;
            }
            ctx.regions.push(Region {
                label: ret_region,
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
    proc: &'a ProcDef,
    statics: &'a [String],
    proc_entry_region: &'a HashMap<String, String>,
) -> ProcScope<'a> {
    let mut vars = HashMap::new();
    for s in statics {
        vars.insert(s.as_str(), VarLoc::Global);
    }
    for (i, p) in proc.params.iter().enumerate() {
        vars.insert(p.as_str(), VarLoc::Param(i));
    }
    for (i, l) in proc.locals.iter().enumerate() {
        vars.insert(l.as_str(), VarLoc::Local(i));
    }
    ProcScope {
        locals_len: proc.locals.len(),
        params_len: proc.params.len(),
        vars,
        proc_entry_region,
    }
}

impl Compiler for ProcToFlowIrCompiler {
    type Source = ProcLangMachine;
    type Target = FlowIrMachine;

    fn compile(source: ProcCode) -> Result<FlowIrCode, String> {
        let mut procs = HashMap::new();
        for p in &source.0.procs {
            if procs.insert(p.name.clone(), p.clone()).is_some() {
                return Err(format!("duplicate procedure in compiler input: {}", p.name));
            }
        }
        let main_proc = procs
            .get("main")
            .ok_or_else(|| "missing main procedure".to_string())?;
        if !main_proc.params.is_empty() {
            return Err("main must have no parameters".to_string());
        }

        let mut proc_entry_region = HashMap::new();
        for p in &source.0.procs {
            proc_entry_region.insert(p.name.clone(), format!("proc_{}", p.name));
        }

        let mut ctx = CompileCtx::default();

        for p in &source.0.procs {
            let scope = build_scope(p, &source.0.statics, &proc_entry_region);
            let normalized = ensure_with_return(p.body.clone());
            let fallthrough = compile_return_region(&[], &scope, &mut ctx)?;
            let body_entry = compile_stmt(&normalized, &fallthrough, &scope, &mut ctx)?;
            let entry_label = proc_entry_region
                .get(&p.name)
                .cloned()
                .ok_or_else(|| format!("internal compiler error: missing entry for {}", p.name))?;

            let mut entry_stmts = Vec::new();
            for _ in 0..p.locals.len() {
                entry_stmts.push(IrStmt::Push {
                    src: ValueExpr::Imm(Number::default()),
                });
            }
            ctx.regions.push(Region {
                label: entry_label,
                blocks: vec![Block {
                    label: "entry".to_string(),
                    stmts: entry_stmts,
                    cont: enter_region(&body_entry),
                }],
            });
        }

        let main_entry = proc_entry_region
            .get("main")
            .cloned()
            .ok_or_else(|| "internal compiler error: missing main entry".to_string())?;

        let startup_region = Region {
            label: "main".to_string(),
            blocks: vec![Block {
                label: "entry".to_string(),
                stmts: vec![IrStmt::Push {
                    src: ValueExpr::CodeLabel("halt".to_string()),
                }],
                cont: enter_region(&main_entry),
            }],
        };
        let halt_region = Region {
            label: "halt".to_string(),
            blocks: vec![Block {
                label: "entry".to_string(),
                stmts: vec![IrStmt::Nop],
                cont: Cont::Halt { ifs: vec![] },
            }],
        };

        let statics = source
            .0
            .statics
            .iter()
            .map(|s| StaticDef {
                label: s.clone(),
                value: Number::default(),
            })
            .collect();

        let mut regions = vec![startup_region, halt_region];
        regions.extend(ctx.regions);
        Ok(FlowIrCode(IrProgram { statics, regions }))
    }

    fn encode_ainput(
        ainput: <Self::Source as Machine>::AInput,
    ) -> Result<<Self::Target as Machine>::AInput, String> {
        let mut entries = std::collections::BTreeMap::new();
        for (name, value) in ainput.vars {
            entries.insert(name, FlowValue::Num(value));
        }
        Ok(StaticEnv { entries })
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
        for (name, value) in output.entries {
            let n = match value {
                FlowValue::Num(n) => n,
                other => {
                    return Err(format!(
                        "proc_lang decode expects numeric static value for {name}, got {}",
                        other.print()
                    ));
                }
            };
            vars.insert(name, n);
        }
        Ok(GlobalEnv { vars })
    }
}
