open Syntax

let ( let* ) = Result.bind

type value =
  | VNat of N.t
  | VBool of bool
  | VUnit
  | VFun of Utils.Identifier.t * expr * env
  | VRec of Utils.Identifier.t * Utils.Identifier.t * expr * env

and env = (Utils.Identifier.t * value) list [@@deriving yojson]

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

let lookup_env env x =
  match List.assoc_opt x env with
  | Some v -> Ok v
  | None -> Error ("unbound variable: " ^ x)

let value_to_expr = function
  | VNat n -> Nat n
  | VBool b -> Bool b
  | VUnit -> Unit
  | VFun (x, b, _) -> Fun (x, b)
  | VRec (f, x, b, _) -> Rec (f, x, b)

let expr_to_value = function
  | Nat n -> Ok (VNat n)
  | Bool b -> Ok (VBool b)
  | Unit -> Ok VUnit
  | Fun _ | Rec _ ->
      Error "plain expr closures cannot be converted without an environment"
  | e -> Error ("expected value expression, found " ^ print e)

let eval_binop a o b =
  let* e = Expr_machine.eval_binop (value_to_expr a) o (value_to_expr b) in
  expr_to_value e

let eval_unop o v =
  let* e = Expr_machine.eval_unop o (value_to_expr v) in
  expr_to_value e

let as_nat = function
  | VNat n -> Ok n
  | _ -> Error "final value is not a natural number"

let value_text v = print (value_to_expr v)

let apply fun_value arg =
  match fun_value with
  | VFun (x, b, env) -> Ok (b, (x, arg) :: env)
  | VRec (f, x, b, env) as r -> Ok (b, (x, arg) :: (f, r) :: env)
  | v -> Error ("application expects a closure, found " ^ value_text v)

let boolean context = function
  | VBool b -> Ok b
  | v -> Error (context ^ " expects a boolean condition, found " ^ value_text v)

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

let parse_code = Syntax.parse_code

let make code inputs =
  Ok { code; state = Eval (apply_inputs code inputs, [], [ Init ]) }

let step s () =
  match s.state with
  | Return (v, [ Init ]) ->
      let* n = as_nat v in
      Ok (Utils.Machine.Halt n)
  | _ ->
      let* state, output = step_state s.state in
      Ok (Utils.Machine.Continue { next = { s with state }; output })

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
  let open Utils.Render in
  let mode, focus, env, k =
    match s.state with
    | Eval (e, env, k) -> ("eval", print e, env, k)
    | Return (v, k) -> ("return", value_text v, env_from_kont k, k)
  in
  state
    [
      text ~title:"code" (print s.code);
      text_table ~title:"state" ~columns:[ "field"; "value" ]
        [ [ "mode"; mode ]; [ "focus"; focus ] ];
      assoc_table ~title:"env" ~columns:[ "name"; "value" ] value_text
        (List.rev env);
      indexed_table ~title:"kont" ~columns:[ "depth"; "frame" ]
        (fun f -> Yojson.Safe.to_string (frame_to_yojson f))
        k;
    ]

module Ainput = Syntax.Ainput
module Rinput = Utils.Text.Unit
module Routput = Syntax.Routput
module Foutput = Syntax.Foutput
