module Via_jump =
  Utils_runtime.Compiler.Compose
    (Recursive_function_to_rec_tm_ir.Main)
    (Rec_tm_ir_to_rec_tm_ir_jump.Main)

include
  Utils_runtime.Compiler.Compose
    (Via_jump)
    (Rec_tm_ir_jump_to_turing_machine.Main)
