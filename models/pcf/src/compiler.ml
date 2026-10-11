let ( let* ) = Result.bind

module Expr_to_cek = struct
  module Source = Expr_machine
  module Target = Cek_machine

  let compile c = Ok c
  let encode_ainput a = Ok a
  let encode_rinput () = Ok ()
  let decode_routput o = Ok o
  let decode_foutput o = Ok o

  let api : Utils.Compiler.api =
    {
      compile_code =
        (fun text ->
          let* code = Source.parse_code text in
          Ok (Syntax.print code));
      encode_ainput =
        (fun text ->
          let* a = Source.Ainput.parse text in
          let* a = encode_ainput a in
          Ok (Target.Ainput.print a));
      encode_rinput =
        (fun text ->
          let* () = Source.Rinput.parse text in
          let* input = encode_rinput () in
          Ok (Target.Rinput.print input));
      decode_routput =
        (fun text ->
          let* o = Target.Routput.parse text in
          let* o = decode_routput o in
          Ok (Source.Routput.print o));
      decode_foutput =
        (fun text ->
          let* o = Target.Foutput.parse text in
          let* o = decode_foutput o in
          Ok (Source.Foutput.print o));
    }
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
    {
      Expr_to_cek.api with
      compile_code =
        (fun text ->
          let* code = Source.parse_code text in
          let* code = compile code in
          Ok (Target.print_code code));
    }
end

let expr_to_cek = Expr_to_cek.api
let expr_to_secd = Expr_to_secd.api
