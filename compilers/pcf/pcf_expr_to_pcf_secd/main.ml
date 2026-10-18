module Source = Pcf_expr.Main
module Target = Pcf_secd.Main
open Pcf_shared.Syntax
open Pcf_secd.Main

let rec compile_expr (e : expr) =
  match e with
  | Nat n -> [ PushNat n ]
  | Bool b -> [ PushBool b ]
  | Unit -> [ PushUnit ]
  | Pcf_shared.Syntax.Print n -> [ Print n ]
  | Var x -> [ Acc x ]
  | Pcf_shared.Syntax.BinOp (a, o, b) ->
      compile_expr a @ compile_expr b @ [ BinOp o ]
  | Pcf_shared.Syntax.UnOp (o, e) -> compile_expr e @ [ UnOp o ]
  | Fun (x, b) -> [ MkCls (x, compile_expr b @ [ Ret ]) ]
  | Rec (f, x, b) -> [ MkRec (f, x, compile_expr b @ [ Ret ]) ]
  | App (f, a) -> compile_expr f @ compile_expr a @ [ Ap ]
  | Pcf_shared.Syntax.If (c, t, e) ->
      compile_expr c @ [ If (compile_expr t, compile_expr e) ]
  | Block _ ->
      invalid_arg "SECD compiler accepts the direct expression language"

let compile code =
  Result.bind (validate_direct code) (fun () ->
      Utils_base.Error.protect (fun () -> compile_expr code))

let encode_ainput a = Ok a
let encode_rinput () = Ok ()
let decode_routput o = Ok o
let decode_foutput o = Ok o
