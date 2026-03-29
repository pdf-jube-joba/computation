mod fn_ptr_parser;
mod fn_ptr_render;
mod mini_prog_parser;
mod mini_prog_render;
pub mod fn_ptr_machine;
pub mod mini_prog_machine;

#[cfg(test)]
mod tests {
    use utils::{Machine, StepResult, TextCodec};

    use crate::{
        fn_ptr_machine::{FnPtrCode, FnPtrMachine},
        mini_prog_machine::{MiniProgCode, MiniProgMachine},
    };

    fn run(source: &str) -> Result<String, String> {
        let code = FnPtrCode::parse(source)?;
        let mut machine = FnPtrMachine::make(code, ())?;
        for _ in 0..100 {
            match machine.step(())? {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output } => return Ok(output),
            }
        }
        Err("machine did not halt within the limit".to_string())
    }

    fn run_full(source: &str, input: Vec<usize>) -> Result<usize, String> {
        let code = MiniProgCode::parse(source)?;
        let mut machine = MiniProgMachine::make(code, input)?;
        for _ in 0..300 {
            match machine.step(())? {
                StepResult::Continue { next, .. } => machine = next,
                StepResult::Halt { output } => return Ok(output),
            }
        }
        Err("machine did not halt within the limit".to_string())
    }

    #[test]
    fn parses_and_runs_assignment() {
        let output = run(
            r#"
            fn main(x) {
                assign x := 3;
                return
            }
            "#,
        )
        .unwrap();
        assert!(output.contains("main"));
        assert!(output.contains("3"));
    }

    #[test]
    fn call_can_mutate_callee_local_through_pointer() {
        let output = run(
            r#"
            fn inc(ptr) {
                assign (ld ptr) #loc := (ld (ld ptr) #loc) + 1;
                return
            }

            fn main(x) {
                assign x := 4;
                call inc(x #addr);
                return
            }
            "#,
        )
        .unwrap();
        assert!(output.contains("5"));
    }

    #[test]
    fn full_machine_can_return_main_argument() {
        let output = run_full(
            r#"
            fn main(x: #num) {
                return ld local x
            }
            "#,
            vec![7],
        )
        .unwrap();
        assert_eq!(output, 7);
    }

    #[test]
    fn full_machine_supports_multi_return_and_call() {
        let output = run_full(
            r#"
            fn pair_up(x: #num, y: #num) {
                return ld local x, ld local y
            }

            fn main(a: #num, b: #num) {
                block (x: #num, y: #num) {
                    call fn pair_up(ld local a, ld local b) -> local x, local y;
                    return ld local x ld local y +
                }
            }
            "#,
            vec![3, 4],
        )
        .unwrap();
        assert_eq!(output, 7);
    }

    #[test]
    fn full_machine_supports_heap_and_block_shadowing() {
        let output = run_full(
            r#"
            fn main(x: #num) {
                block (p: #ptr) {
                    halloc #num -> local p;
                    assign (ld local p) #loc := ld local x;
                    block (x: #num) {
                        assign local x := 0;
                        return ld (ld local p) #loc
                    }
                }
            }
            "#,
            vec![9],
        )
        .unwrap();
        assert_eq!(output, 9);
    }
}
