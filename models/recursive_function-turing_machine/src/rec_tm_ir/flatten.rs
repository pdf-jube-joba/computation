use super::machine::Program;

pub fn flatten_program(program: &Program) -> Result<Program, String> {
    crate::rec_tm_ir_jump::flatten_program(program)
}
