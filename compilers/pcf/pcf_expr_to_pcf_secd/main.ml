module Source = Pcf_expr.Main
module Target = Pcf_secd.Main

let compile code =
  Utils.Json.protect (fun () -> Pcf_secd.Main.compile_expr code)

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
