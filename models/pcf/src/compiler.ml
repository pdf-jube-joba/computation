module Expr_to_cek = struct
  module Source = Expr_machine
  module Target = Cek_machine

  let compile c = Ok c
  let encode_ainput a = Ok a
  let encode_rinput () = Ok ()
  let decode_routput o = Ok o
  let decode_foutput o = Ok o

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

module Expr_to_secd = struct
  module Source = Expr_machine
  module Target = Secd_machine

  let compile code =
    Utils.Json.protect (fun () -> Secd_machine.compile_expr code)

  let encode_ainput a = Ok a
  let encode_rinput () = Ok ()
  let decode_routput o = Ok o
  let decode_foutput o = Ok o

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
