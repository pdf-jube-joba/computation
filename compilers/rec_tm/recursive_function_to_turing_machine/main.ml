module Source = Recursive_function.Main
module Target = Turing_machine.Main

let encode_ainput = Recursive_function_to_rec_tm_ir.Main.encode_ainput
let encode_rinput = Recursive_function_to_rec_tm_ir.Main.encode_rinput
let decode_routput = Recursive_function_to_rec_tm_ir.Main.decode_routput
let decode_foutput = Recursive_function_to_rec_tm_ir.Main.decode_foutput

let compile f =
  let open Utils.Machine in
  let* ir = Recursive_function_to_rec_tm_ir.Main.compile f in
  let* jump = Rec_tm_ir_to_rec_tm_ir_jump.Main.compile ir in
  Rec_tm_ir_jump_to_turing_machine.Main.compile jump

let api =
  Utils.Compiler.make_api
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
