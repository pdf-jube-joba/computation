open Pcf_shared.Syntax
module Code = Pcf_shared.Code
open Pcf_shared.Closure_value

type frame =
  | BinOpLeft of binop * expr * env
  | BinOpRight of value * binop
  | Unary of unop
  | AppFun of expr * env
  | AppArg of value
  | IfFrame of expr * expr * env
  | Init
[@@deriving yojson]

type cek_state =
  | Eval of expr * env * frame list
  | Return of value * frame list
[@@deriving yojson]

let step_state = function
  | Eval (e, env, k) -> (
      match e with
      | Nat n -> Ok (Return (VNat n, k), None)
      | Bool b -> Ok (Return (VBool b, k), None)
      | Unit -> Ok (Return (VUnit, k), None)
      | Print n -> Ok (Return (VUnit, k), Some n)
      | Var x ->
          let* v = lookup_env env x in
          Ok (Return (v, k), None)
      | Fun (x, b) -> Ok (Return (VFun (x, b, env), k), None)
      | Rec (f, x, b) -> Ok (Return (VRec (f, x, b, env), k), None)
      | BinOp (a, o, b) -> Ok (Eval (a, env, BinOpLeft (o, b, env) :: k), None)
      | UnOp (o, e) -> Ok (Eval (e, env, Unary o :: k), None)
      | App (f, a) -> Ok (Eval (f, env, AppFun (a, env) :: k), None)
      | If (c, t, e) -> Ok (Eval (c, env, IfFrame (t, e, env) :: k), None)
      | Block _ -> Error "blocks belong to the expression/statement machine")
  | Return (_, []) -> Error "empty continuation"
  | Return (v, frame :: k) -> (
      match frame with
      | Init -> Ok (Return (v, Init :: k), None)
      | BinOpLeft (o, b, env) -> Ok (Eval (b, env, BinOpRight (v, o) :: k), None)
      | BinOpRight (a, o) ->
          let* v = eval_binop a o v in
          Ok (Return (v, k), None)
      | Unary o ->
          let* v = eval_unop o v in
          Ok (Return (v, k), None)
      | AppFun (a, env) -> Ok (Eval (a, env, AppArg v :: k), None)
      | AppArg f ->
          let* b, env = apply f v in
          Ok (Eval (b, env, k), None)
      | IfFrame (t, e, env) ->
          let* b = boolean "if" v in
          Ok (Eval ((if b then t else e), env, k), None))

type code = expr
type ainput = N.t list
type rinput = unit
type routput = print_effect
type foutput = N.t
type state = { code : expr; state : cek_state } [@@deriving yojson]

let make code inputs =
  let* () = validate_direct code in
  Ok { code; state = Eval (apply_inputs code inputs, [], [ Init ]) }

let step s () =
  match s.state with
  | Return (v, [ Init ]) ->
      let* n = as_nat v in
      Ok (Utils_runtime.Machine.Halt n)
  | _ ->
      let* state, output = step_state s.state in
      Ok (Utils_runtime.Machine.Continue { next = { s with state }; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

let env_from_kont k =
  List.find_map
    (function
      | BinOpLeft (_, _, env) | AppFun (_, env) | IfFrame (_, _, env) ->
          Some env
      | _ -> None)
    k
  |> Option.value ~default:[]

let render s =
  let open Utils_runtime.Render in
  let mode, focus, env, k =
    match s.state with
    | Eval (e, env, k) -> ("eval", Code.print e, env, k)
    | Return (v, k) -> ("return", value_text v, env_from_kont k, k)
  in
  state
    [
      text ~title:"code" (Code.print s.code);
      text_table ~title:"state" ~columns:[ "field"; "value" ]
        [ [ "mode"; mode ]; [ "focus"; focus ] ];
      keyed_table ~key_text:Utils_names.Identifier.to_string ~title:"env"
        ~columns:[ "name"; "value" ] value_text (List.rev env);
      indexed_table ~title:"kont" ~columns:[ "depth"; "frame" ]
        (fun f -> Yojson.Safe.to_string (frame_to_yojson f))
        k;
    ]

module Ainput = Pcf_shared.Io.Ainput
module Rinput = Utils_codec.Text.Unit
module Routput = Pcf_shared.Io.Routput
module Foutput = Pcf_shared.Io.Foutput
