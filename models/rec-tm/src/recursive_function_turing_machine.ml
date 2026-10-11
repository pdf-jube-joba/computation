module Rec_tm_ir = Rec_tm_ir
module Rec_tm_ir_jump = Rec_tm_ir_jump
module Compile_jump = Compile_jump
module Compile_tm = Compile_tm
module Rec_to_ir = Rec_to_ir

module Rec_to_tm = struct
  module Source = Recursive_function
  module Target = Turing_machine.Machine

  let encode_ainput = Rec_to_ir.encode_ainput
  let encode_rinput = Rec_to_ir.encode_rinput
  let decode_routput = Rec_to_ir.decode_routput
  let decode_foutput = Rec_to_ir.decode_foutput

  let compile f =
    let open Utils.Machine in
    let* ir = Rec_to_ir.compile f in
    let* jump = Compile_jump.compile ir in
    Compile_tm.compile jump

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
end
