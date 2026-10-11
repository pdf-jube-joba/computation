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
          let* a = Source.parse_ainput text in
          Ok (Syntax.print_ainput a));
      encode_rinput =
        (fun text ->
          let* () = Source.parse_rinput text in
          Ok "");
      decode_routput =
        (fun text ->
          let* o = Syntax.parse_effect text in
          Ok (Syntax.print_effect o));
      decode_foutput =
        (fun text ->
          let* o = Syntax.parse_natural_text text in
          Ok (Utils.Number.to_string o));
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
