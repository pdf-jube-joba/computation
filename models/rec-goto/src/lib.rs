use std::collections::HashMap;

use goto_lang::machine::{Code, Command, Environment, Program};
use recursive_function::machine::RecursiveFunctions;
use utils::identifier::Identifier;
use utils::number::Number;
use utils::{Compiler, Machine};

const OUTPUT_VAR: &str = "y";
const ALWAYS_ONE_VAR: &str = "__one";
const INPUT_VAR_PREFIX: &str = "x";
const TEMP_VAR_PREFIX: &str = "__tmp";

#[derive(Clone, Copy)]
struct Label(usize);

enum UnresolvedCommand {
    Clr(Identifier),
    Inc(Identifier),
    Dec(Identifier),
    Cpy(Identifier, Identifier),
    Ifnz(Identifier, Label),
}

impl UnresolvedCommand {
    fn resolve(self, labels: &HashMap<usize, usize>) -> Result<Command, String> {
        match self {
            UnresolvedCommand::Clr(var) => Ok(Command::Clr(var)),
            UnresolvedCommand::Inc(var) => Ok(Command::Inc(var)),
            UnresolvedCommand::Dec(var) => Ok(Command::Dec(var)),
            UnresolvedCommand::Cpy(dst, src) => Ok(Command::Cpy(dst, src)),
            UnresolvedCommand::Ifnz(var, label) => {
                let target = labels
                    .get(&label.0)
                    .ok_or_else(|| format!("undefined label: {}", label.0))?;
                Ok(Command::Ifnz(var, Number::from(*target)))
            }
        }
    }
}

struct CodeBuilder {
    commands: Vec<UnresolvedCommand>,
    labels: HashMap<usize, usize>,
    next_label: usize,
    temp_counter: usize,
}

impl CodeBuilder {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
            labels: HashMap::new(),
            next_label: 0,
            temp_counter: 0,
        }
    }

    fn new_label(&mut self) -> Label {
        let label = Label(self.next_label);
        self.next_label += 1;
        label
    }

    fn mark(&mut self, label: Label) {
        self.labels.insert(label.0, self.commands.len());
    }

    fn temp_var(&mut self) -> Identifier {
        let name = format!("{TEMP_VAR_PREFIX}{}", self.temp_counter);
        self.temp_counter += 1;
        id(name)
    }

    fn emit(&mut self, command: UnresolvedCommand) {
        self.commands.push(command);
    }

    fn emit_clr(&mut self, var: &Identifier) {
        self.emit(UnresolvedCommand::Clr(var.clone()));
    }

    fn emit_inc(&mut self, var: &Identifier) {
        self.emit(UnresolvedCommand::Inc(var.clone()));
    }

    fn emit_dec(&mut self, var: &Identifier) {
        self.emit(UnresolvedCommand::Dec(var.clone()));
    }

    fn emit_cpy(&mut self, dst: &Identifier, src: &Identifier) {
        self.emit(UnresolvedCommand::Cpy(dst.clone(), src.clone()));
    }

    fn emit_ifnz(&mut self, var: &Identifier, target: Label) {
        self.emit(UnresolvedCommand::Ifnz(var.clone(), target));
    }

    fn emit_goto(&mut self, target: Label) {
        self.emit_ifnz(&id(ALWAYS_ONE_VAR), target);
    }

    fn compile_recursive_function(
        &mut self,
        function: &RecursiveFunctions,
        args: &[Identifier],
        dst: &Identifier,
    ) {
        match function {
            RecursiveFunctions::ZeroConstant => {
                self.emit_clr(dst);
            }
            RecursiveFunctions::Successor => {
                self.emit_cpy(dst, &args[0]);
                self.emit_inc(dst);
            }
            RecursiveFunctions::Projection { projection_num, .. } => {
                self.emit_cpy(dst, &args[*projection_num]);
            }
            RecursiveFunctions::Composition {
                outer_func,
                inner_funcs,
                ..
            } => {
                let outer_args: Vec<Identifier> = inner_funcs
                    .iter()
                    .map(|inner| {
                        let tmp = self.temp_var();
                        self.compile_recursive_function(inner, args, &tmp);
                        tmp
                    })
                    .collect();
                self.compile_recursive_function(outer_func.as_ref(), &outer_args, dst);
            }
            RecursiveFunctions::PrimitiveRecursion {
                zero_func,
                succ_func,
            } => self.compile_primitive_recursion(
                zero_func.as_ref(),
                succ_func.as_ref(),
                args,
                dst,
            ),
            RecursiveFunctions::MuOperator { mu_func } => {
                self.compile_mu_operator(mu_func.as_ref(), args, dst);
            }
        }
    }

    fn compile_primitive_recursion(
        &mut self,
        zero_func: &RecursiveFunctions,
        succ_func: &RecursiveFunctions,
        args: &[Identifier],
        dst: &Identifier,
    ) {
        let first = args[0].clone();
        let rest = args[1..].to_vec();

        let result = self.temp_var();
        let counter = self.temp_var();
        let index = self.temp_var();
        let succ_value = self.temp_var();

        self.compile_recursive_function(zero_func, &rest, &result);
        self.emit_cpy(&counter, &first);
        self.emit_clr(&index);

        let loop_cond = self.new_label();
        let loop_body = self.new_label();
        let loop_end = self.new_label();

        self.mark(loop_cond);
        self.emit_ifnz(&counter, loop_body);
        self.emit_goto(loop_end);

        self.mark(loop_body);
        let mut succ_args = vec![result.clone(), index.clone()];
        succ_args.extend(rest.iter().cloned());
        self.compile_recursive_function(succ_func, &succ_args, &succ_value);
        self.emit_cpy(&result, &succ_value);
        self.emit_inc(&index);
        self.emit_dec(&counter);
        self.emit_goto(loop_cond);

        self.mark(loop_end);
        self.emit_cpy(dst, &result);
    }

    fn compile_mu_operator(
        &mut self,
        mu_func: &RecursiveFunctions,
        args: &[Identifier],
        dst: &Identifier,
    ) {
        let index = self.temp_var();
        let value = self.temp_var();

        self.emit_clr(&index);

        let loop_start = self.new_label();
        let loop_continue = self.new_label();
        let loop_end = self.new_label();

        self.mark(loop_start);
        let mut mu_args = vec![index.clone()];
        mu_args.extend(args.iter().cloned());
        self.compile_recursive_function(mu_func, &mu_args, &value);
        self.emit_ifnz(&value, loop_continue);
        self.emit_cpy(dst, &index);
        self.emit_goto(loop_end);

        self.mark(loop_continue);
        self.emit_inc(&index);
        self.emit_goto(loop_start);

        self.mark(loop_end);
    }

    fn finalize(self) -> Result<Code, String> {
        let commands = self
            .commands
            .into_iter()
            .map(|cmd| cmd.resolve(&self.labels))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Code(commands))
    }
}

fn id(name: impl AsRef<str>) -> Identifier {
    Identifier::new(name.as_ref()).unwrap()
}

fn input_vars(arity: usize) -> Vec<Identifier> {
    (0..arity)
        .map(|index| id(format!("{INPUT_VAR_PREFIX}{index}")))
        .collect()
}

pub fn compile(function: &RecursiveFunctions) -> Result<Code, String> {
    let mut builder = CodeBuilder::new();
    builder.emit_clr(&id(ALWAYS_ONE_VAR));
    builder.emit_inc(&id(ALWAYS_ONE_VAR));

    let args = input_vars(function.parameter_length());
    builder.compile_recursive_function(function, &args, &id(OUTPUT_VAR));
    builder.finalize()
}

pub struct RecToGotoCompiler;

impl Compiler for RecToGotoCompiler {
    type Source = recursive_function::machine::Program;
    type Target = Program;

    fn compile(
        source: <<Self as Compiler>::Source as Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as Machine>::Code, String> {
        compile(&source)
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as Machine>::AInput, String> {
        let env = ainput
            .into_iter()
            .enumerate()
            .map(|(index, value)| (id(format!("{INPUT_VAR_PREFIX}{index}")), value))
            .collect();
        Ok(Environment { env })
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
        Ok(output.get(&id(OUTPUT_VAR)))
    }
}

#[cfg(test)]
mod tests {
    use recursive_function::machine::{Program as RecProgram, RecursiveFunctions};
    use utils::StepResult;

    use super::*;

    fn run_rec(function: RecursiveFunctions, input: Vec<usize>) -> Number {
        let input: Vec<Number> = input.into_iter().map(Number::from).collect();
        let mut machine = RecProgram::make(function.clone(), input).unwrap();
        loop {
            match machine.step(()).unwrap() {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output, .. } => return output,
            }
        }
    }

    fn run_goto(function: RecursiveFunctions, input: Vec<usize>, step_limit: usize) -> Number {
        let code = compile(&function).unwrap();
        let ainput = RecToGotoCompiler::encode_ainput(input.into_iter().map(Number::from).collect())
            .unwrap();
        let mut machine = Program::make(code, ainput).unwrap();
        for _ in 0..step_limit {
            match machine.step(()).unwrap() {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output, .. } => {
                    return RecToGotoCompiler::decode_foutput(output).unwrap();
                }
            }
        }
        panic!("step limit exceeded");
    }

    #[test]
    fn compile_successor() {
        let function = RecursiveFunctions::succ();
        assert_eq!(run_rec(function.clone(), vec![3]), run_goto(function, vec![3], 1000));
    }

    #[test]
    fn compile_composition() {
        let function = RecursiveFunctions::composition(
            RecursiveFunctions::succ(),
            vec![RecursiveFunctions::succ()],
        )
        .unwrap();
        assert_eq!(run_rec(function.clone(), vec![5]), run_goto(function, vec![5], 2000));
    }

    #[test]
    fn compile_primitive_recursion_add() {
        let add = RecursiveFunctions::primitive_recursion(
            RecursiveFunctions::projection(1, 0).unwrap(),
            RecursiveFunctions::composition(
                RecursiveFunctions::succ(),
                vec![RecursiveFunctions::projection(3, 0).unwrap()],
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(run_rec(add.clone(), vec![2, 4]), run_goto(add, vec![2, 4], 5000));
    }

    #[test]
    fn compile_mu_operator() {
        let pred = RecursiveFunctions::projection(2, 0).unwrap();
        let mu = RecursiveFunctions::muoperator(pred).unwrap();
        assert_eq!(run_rec(mu.clone(), vec![3]), run_goto(mu, vec![3], 5000));
    }
}
