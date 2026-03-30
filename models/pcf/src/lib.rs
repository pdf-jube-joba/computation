pub mod cek_machine;
pub mod compiler;
pub mod expr_machine;
pub mod expr_stmt_machine;
pub mod secd_machine;
pub mod syntax;

mod expr_stmt_parser;
mod syntax_parser;

#[cfg(test)]
mod tests {
    use utils::{Compiler, Machine, StepResult, TextCodec};

    use crate::{
        cek_machine::CekMachine,
        compiler::{ExprToCekCompiler, ExprToSecdCompiler},
        expr_machine::ExprMachine,
        expr_stmt_machine::{ExprStmtCode, ExprStmtMachine},
        secd_machine::SecdMachine,
        syntax::{ExprCode, PrintEffect},
    };

    fn run_machine<M>(code: M::Code, ainput: M::AInput) -> Result<(Vec<usize>, M::FOutput), String>
    where
        M: Machine<ROutput = PrintEffect, RInput = (), AInput = Vec<usize>>,
    {
        let mut machine = M::make(code, ainput)?;
        let mut outputs = Vec::new();
        for _ in 0..200 {
            match machine.step(())? {
                StepResult::Continue { next, output } => {
                    if let Some(value) = output.0 {
                        outputs.push(value);
                    }
                    machine = next;
                }
                StepResult::Halt { output } => return Ok((outputs, output)),
            }
        }
        Err("machine did not halt within the limit".to_string())
    }

    #[test]
    fn expr_cek_and_secd_preserve_print_order() {
        let source = ExprCode::parse("(fun x => (fun y => 7))(print 1)(print 2)").unwrap();

        let expr_result = run_machine::<ExprMachine>(source.clone(), vec![]).unwrap();
        assert_eq!(expr_result, (vec![1, 2], 7));

        let cek_code = ExprToCekCompiler::compile(source.clone()).unwrap();
        let cek_result = run_machine::<CekMachine>(cek_code, vec![]).unwrap();
        assert_eq!(cek_result, expr_result);

        let secd_code = ExprToSecdCompiler::compile(source).unwrap();
        let secd_result = run_machine::<SecdMachine>(secd_code, vec![]).unwrap();
        assert_eq!(secd_result, expr_result);
    }

    #[test]
    fn recursive_function_application_works_across_all_models() {
        let source = ExprCode::parse("rec f x => if #true then x else f(x) fi").unwrap();

        let expr_result = run_machine::<ExprMachine>(source.clone(), vec![5]).unwrap();
        assert_eq!(expr_result, (vec![], 5));

        let cek_result =
            run_machine::<CekMachine>(ExprToCekCompiler::compile(source.clone()).unwrap(), vec![5])
                .unwrap();
        assert_eq!(cek_result, expr_result);

        let secd_result =
            run_machine::<SecdMachine>(ExprToSecdCompiler::compile(source).unwrap(), vec![5])
                .unwrap();
        assert_eq!(secd_result, expr_result);
    }

    #[test]
    fn expr_stmt_block_preserves_print_order() {
        let source = ExprStmtCode::parse("{ print 1; 2 } + { print 3; 4 }").unwrap();
        let result = run_machine::<ExprStmtMachine>(source, vec![]).unwrap();
        assert_eq!(result, (vec![1, 3], 6));
    }

    #[test]
    fn expr_stmt_closure_captures_block_local_binding() {
        let source = ExprStmtCode::parse("({ let x := 1; fun y => x + y })(2)").unwrap();
        let result = run_machine::<ExprStmtMachine>(source, vec![]).unwrap();
        assert_eq!(result, (vec![], 3));
    }
}
