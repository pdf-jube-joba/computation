module Via_jump =
  Utils_runtime.Compiler.Compose
    (Recursive_function_to_rec_tm_ir.Main)
    (Rec_tm_ir_to_rec_tm_ir_jump.Main)

include
  Utils_runtime.Compiler.Compose
    (Via_jump)
    (Rec_tm_ir_jump_to_turing_machine.Main)

let api =
  Utils_runtime.Compiler_text_api.make
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
