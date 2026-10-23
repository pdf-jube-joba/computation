module Source = Pcf_expr.Main
module Target = Pcf_cek.Main

(* CEK executes the source AST directly. This boundary validates the base
   language; CEK's initial configuration and transitions belong to the model. *)
let compile (code : Source.Code.t) : (Target.Code.t, string) result =
  Result.map (fun () -> code) (Pcf_shared.Syntax.validate_direct code)

let encode_ainput a = Ok a
let encode_rinput () = Ok ()
let decode_routput o = Ok o
let decode_foutput o = Ok o
