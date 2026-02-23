use std::collections::HashMap;

use turing_machine::machine::{CodeEntry, Direction, Sign, State, TuringMachineDefinition};
use utils::{Compiler, Machine, TextCodec};

use super::machine::{LValue, Program, RValue, Stmt};

pub struct RecTmIrJumpToTmCompiler;

impl Compiler for RecTmIrJumpToTmCompiler {
    type Source = crate::rec_tm_ir_jump::RecTmIrJumpMachine;
    type Target = turing_machine::machine::TuringMachine;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        compile_program(&source)
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

fn compile_program(program: &Program) -> Result<TuringMachineDefinition, String> {
    let alphabet = normalized_alphabet(&program.alphabet)?;
    validate_constants(program, &alphabet)?;
    let vars = collect_vars(&program.body);
    let envs = enumerate_envs(&alphabet, &vars)?;
    let state_map = build_state_map(program.body.len(), &envs)?;

    let mut code = Vec::new();
    for (pc, stmt) in program.body.iter().enumerate() {
        for (env_idx, env) in envs.iter().enumerate() {
            let state = state_for(pc, env_idx, &state_map)?;
            match stmt {
                Stmt::Lt => add_for_all_tape(
                    &mut code,
                    &alphabet,
                    &state,
                    pc + 1,
                    env_idx,
                    &state_map,
                    |sign| (sign.clone(), Direction::Left),
                )?,
                Stmt::Rt => add_for_all_tape(
                    &mut code,
                    &alphabet,
                    &state,
                    pc + 1,
                    env_idx,
                    &state_map,
                    |sign| (sign.clone(), Direction::Right),
                )?,
                Stmt::Assign { dst, src } => match dst {
                    LValue::Var(name) => {
                        let dst_idx = vars
                            .iter()
                            .position(|v| v == name)
                            .ok_or_else(|| format!("Unknown variable '{}'", name))?;
                        if rvalue_depends_on_head(src) {
                            for sign in &alphabet {
                                let value = eval_rvalue(src, &vars, env, sign)?;
                                let mut next_env = env.clone();
                                next_env[dst_idx] = value;
                                let next_env_idx = env_index(&envs, &next_env)?;
                                let next_state = state_for(pc + 1, next_env_idx, &state_map)?;
                                code.push((
                                    (sign.clone(), state.clone()),
                                    (sign.clone(), next_state, Direction::Constant),
                                ));
                            }
                        } else {
                            let value = eval_rvalue_no_head(src, &vars, env)?;
                            let mut next_env = env.clone();
                            next_env[dst_idx] = value;
                            let next_env_idx = env_index(&envs, &next_env)?;
                            add_for_all_tape(
                                &mut code,
                                &alphabet,
                                &state,
                                pc + 1,
                                next_env_idx,
                                &state_map,
                                |sign| (sign.clone(), Direction::Constant),
                            )?;
                        }
                    }
                    LValue::Head => {
                        if rvalue_depends_on_head(src) {
                            add_for_all_tape(
                                &mut code,
                                &alphabet,
                                &state,
                                pc + 1,
                                env_idx,
                                &state_map,
                                |sign| (sign.clone(), Direction::Constant),
                            )?;
                        } else {
                            let value = eval_rvalue_no_head(src, &vars, env)?;
                            add_for_all_tape(
                                &mut code,
                                &alphabet,
                                &state,
                                pc + 1,
                                env_idx,
                                &state_map,
                                |_| (value.clone(), Direction::Constant),
                            )?;
                        }
                    }
                },
                Stmt::Jump { target, cond } => {
                    if let Some(cond) = cond {
                        let head_dep = rvalue_depends_on_head(&cond.left)
                            || rvalue_depends_on_head(&cond.right);
                        if head_dep {
                            for sign in &alphabet {
                                let left = eval_rvalue(&cond.left, &vars, env, sign)?;
                                let right = eval_rvalue(&cond.right, &vars, env, sign)?;
                                let next_pc = if left == right { *target } else { pc + 1 };
                                let next_state = state_for(next_pc, env_idx, &state_map)?;
                                code.push((
                                    (sign.clone(), state.clone()),
                                    (sign.clone(), next_state, Direction::Constant),
                                ));
                            }
                        } else {
                            let left = eval_rvalue_no_head(&cond.left, &vars, env)?;
                            let right = eval_rvalue_no_head(&cond.right, &vars, env)?;
                            let next_pc = if left == right { *target } else { pc + 1 };
                            add_for_all_tape(
                                &mut code,
                                &alphabet,
                                &state,
                                next_pc,
                                env_idx,
                                &state_map,
                                |sign| (sign.clone(), Direction::Constant),
                            )?;
                        }
                    } else {
                        add_for_all_tape(
                            &mut code,
                            &alphabet,
                            &state,
                            *target,
                            env_idx,
                            &state_map,
                            |sign| (sign.clone(), Direction::Constant),
                        )?;
                    }
                }
            }
        }
    }

    let init_state = state_for(0, 0, &state_map)?;
    let accepted_states = (0..envs.len())
        .map(|env_idx| state_for(program.body.len(), env_idx, &state_map))
        .collect::<Result<Vec<_>, _>>()?;
    TuringMachineDefinition::new(init_state, accepted_states, code).map_err(|e| e.to_string())
}

fn validate_constants(program: &Program, alphabet: &[Sign]) -> Result<(), String> {
    for stmt in &program.body {
        match stmt {
            Stmt::Assign { dst: _, src } => validate_rvalue_const(src, alphabet)?,
            Stmt::Jump {
                cond: Some(cond), ..
            } => {
                validate_rvalue_const(&cond.left, alphabet)?;
                validate_rvalue_const(&cond.right, alphabet)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn normalized_alphabet(alphabet: &[Sign]) -> Result<Vec<Sign>, String> {
    if alphabet.is_empty() {
        return Err("alphabet must not be empty".to_string());
    }
    let mut set = HashMap::<Sign, ()>::new();
    let mut out = Vec::new();
    for sign in alphabet {
        if !set.contains_key(sign) {
            set.insert(sign.clone(), ());
            out.push(sign.clone());
        }
    }
    if !out.iter().any(|sign| *sign == Sign::blank()) {
        out.push(Sign::blank());
    }
    Ok(out)
}

fn collect_vars(stmts: &[Stmt]) -> Vec<String> {
    let mut set = HashMap::new();
    for stmt in stmts {
        match stmt {
            Stmt::Assign { dst, src } => {
                if let LValue::Var(name) = dst {
                    set.entry(name.clone()).or_insert(());
                }
                if let RValue::Var(name) = src {
                    set.entry(name.clone()).or_insert(());
                }
            }
            Stmt::Jump {
                cond: Some(cond), ..
            } => {
                if let RValue::Var(name) = &cond.left {
                    set.entry(name.clone()).or_insert(());
                }
                if let RValue::Var(name) = &cond.right {
                    set.entry(name.clone()).or_insert(());
                }
            }
            _ => {}
        }
    }
    let mut vars: Vec<String> = set.into_keys().collect();
    vars.sort();
    vars
}

fn validate_rvalue_const(value: &RValue, alphabet: &[Sign]) -> Result<(), String> {
    if let RValue::Const(sign) = value
        && !alphabet.contains(sign)
    {
        return Err(format!("Unknown sign in const: {}", sign.print()));
    }
    Ok(())
}

fn rvalue_depends_on_head(value: &RValue) -> bool {
    matches!(value, RValue::Head)
}

fn eval_rvalue(value: &RValue, vars: &[String], env: &[Sign], head: &Sign) -> Result<Sign, String> {
    match value {
        RValue::Var(name) => vars
            .iter()
            .position(|v| v == name)
            .map(|idx| env[idx].clone())
            .ok_or_else(|| format!("Unknown variable '{}'", name)),
        RValue::Head => Ok(head.clone()),
        RValue::Const(sign) => Ok(sign.clone()),
    }
}

fn eval_rvalue_no_head(value: &RValue, vars: &[String], env: &[Sign]) -> Result<Sign, String> {
    match value {
        RValue::Var(name) => vars
            .iter()
            .position(|v| v == name)
            .map(|idx| env[idx].clone())
            .ok_or_else(|| format!("Unknown variable '{}'", name)),
        RValue::Head => Err("Head value depends on tape".to_string()),
        RValue::Const(sign) => Ok(sign.clone()),
    }
}

fn enumerate_envs(alphabet: &[Sign], vars: &[String]) -> Result<Vec<Vec<Sign>>, String> {
    let mut out = Vec::new();
    if vars.is_empty() {
        out.push(Vec::new());
        return Ok(out);
    }
    fn build(
        idx: usize,
        vars: &[String],
        alphabet: &[Sign],
        current: &mut Vec<Sign>,
        out: &mut Vec<Vec<Sign>>,
    ) {
        if idx == vars.len() {
            out.push(current.clone());
            return;
        }
        for sign in alphabet {
            current.push(sign.clone());
            build(idx + 1, vars, alphabet, current, out);
            current.pop();
        }
    }
    let mut current = Vec::new();
    build(0, vars, alphabet, &mut current, &mut out);
    Ok(out)
}

fn build_state_map(
    pc_len: usize,
    envs: &[Vec<Sign>],
) -> Result<HashMap<(usize, usize), State>, String> {
    let mut map = HashMap::new();
    for pc in 0..=pc_len {
        for env_idx in 0..envs.len() {
            let name = format!("q_{}_{}", pc, env_idx);
            let state = turing_machine::machine::State::try_from(&name)?;
            map.insert((pc, env_idx), state);
        }
    }
    Ok(map)
}

fn state_for(
    pc: usize,
    env_idx: usize,
    map: &HashMap<(usize, usize), State>,
) -> Result<State, String> {
    map.get(&(pc, env_idx))
        .cloned()
        .ok_or_else(|| format!("State missing for pc {}, env {}", pc, env_idx))
}

fn env_index(envs: &[Vec<Sign>], env: &[Sign]) -> Result<usize, String> {
    envs.iter()
        .position(|candidate| candidate == env)
        .ok_or_else(|| "Environment not found".to_string())
}

fn add_for_all_tape<F>(
    code: &mut Vec<CodeEntry>,
    alphabet: &[Sign],
    state: &State,
    next_pc: usize,
    next_env_idx: usize,
    map: &HashMap<(usize, usize), State>,
    mut next: F,
) -> Result<(), String>
where
    F: FnMut(&Sign) -> (Sign, Direction),
{
    let next_state = state_for(next_pc, next_env_idx, map)?;
    for sign in alphabet {
        let (next_sign, dir) = next(sign);
        code.push((
            (sign.clone(), state.clone()),
            (next_sign, next_state.clone(), dir),
        ));
    }
    Ok(())
}
