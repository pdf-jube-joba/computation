open Pcf_shared.Syntax
open Pcf_shared.Closure_value
module Code = Code

type value = Pcf_shared.Closure_value.value [@@deriving yojson]
type env = Pcf_shared.Closure_value.env [@@deriving yojson]

type frame =
  | BinOpLeft of binop * expr * env
  | BinOpRight of value * binop
  | Unary of unop
  | AppFun of expr * env
  | AppArg of value
  | IfExpr of expr * expr * env
  | LetFrame of Utils_names.Identifier.t * env
  | SeqFrame of stmt
  | IfStmtFrame of stmt * env
  | WhileFrame of expr * stmt * env
  | ScopeOut of expr
  | Init
[@@deriving yojson]

type phase =
  | EvalExpr of expr * env * frame list
  | Return of value * frame list
  | EvalStmt of stmt * env * frame list
  | Done of env * frame list
[@@deriving yojson]

let step_state = function
  | EvalExpr (e, env, k) -> (
      match e with
      | Nat n -> Ok (Return (VNat n, k), None)
      | Bool b -> Ok (Return (VBool b, k), None)
      | Unit -> Ok (Return (VUnit, k), None)
      | Var x ->
          let* v = lookup_env env x in
          Ok (Return (v, k), None)
      | Fun (x, b) -> Ok (Return (VFun (x, b, env), k), None)
      | Rec (f, x, b) -> Ok (Return (VRec (f, x, b, env), k), None)
      | BinOp (a, o, b) ->
          Ok (EvalExpr (a, env, BinOpLeft (o, b, env) :: k), None)
      | UnOp (o, e) -> Ok (EvalExpr (e, env, Unary o :: k), None)
      | App (f, a) -> Ok (EvalExpr (f, env, AppFun (a, env) :: k), None)
      | If (c, t, e) -> Ok (EvalExpr (c, env, IfExpr (t, e, env) :: k), None)
      | Block (s, e) -> Ok (EvalStmt (s, env, ScopeOut e :: k), None)
      | Print _ ->
          Error "print is a statement in the expression/statement language")
  | Return (_, []) | Done (_, []) -> Error "empty continuation"
  | Return (v, frame :: k) -> (
      match frame with
      | Init -> Ok (Return (v, Init :: k), None)
      | BinOpLeft (o, b, env) ->
          Ok (EvalExpr (b, env, BinOpRight (v, o) :: k), None)
      | BinOpRight (a, o) ->
          let* v = eval_binop a o v in
          Ok (Return (v, k), None)
      | Unary o ->
          let* v = eval_unop o v in
          Ok (Return (v, k), None)
      | AppFun (a, env) -> Ok (EvalExpr (a, env, AppArg v :: k), None)
      | AppArg f ->
          let* b, env = apply f v in
          Ok (EvalExpr (b, env, k), None)
      | IfExpr (t, e, env) ->
          let* b = boolean "if" v in
          Ok (EvalExpr ((if b then t else e), env, k), None)
      | LetFrame (x, env) -> Ok (Done ((x, v) :: env, k), None)
      | IfStmtFrame (s, env) ->
          let* b = boolean "if" v in
          Ok ((if b then EvalStmt (s, env, k) else Done (env, k)), None)
      | WhileFrame (c, s, env) ->
          let* b = boolean "while" v in
          Ok
            ( (if b then EvalStmt (s, env, SeqFrame (While (c, s)) :: k)
               else Done (env, k)),
              None )
      | SeqFrame _ | ScopeOut _ ->
          Error "internal CEK error: statement frame during return")
  | EvalStmt (s, env, k) -> (
      match s with
      | Skip -> Ok (Done (env, k), None)
      | PrintStmt n -> Ok (Done (env, k), Some n)
      | Seq (a, b) -> Ok (EvalStmt (a, env, SeqFrame b :: k), None)
      | Let (x, e) -> Ok (EvalExpr (e, env, LetFrame (x, env) :: k), None)
      | IfStmt (c, s) -> Ok (EvalExpr (c, env, IfStmtFrame (s, env) :: k), None)
      | While (c, s) -> Ok (EvalExpr (c, env, WhileFrame (c, s, env) :: k), None)
      )
  | Done (env, frame :: k) -> (
      match frame with
      | SeqFrame s -> Ok (EvalStmt (s, env, k), None)
      | ScopeOut e -> Ok (EvalExpr (e, env, k), None)
      | Init -> Ok (Done (env, Init :: k), None)
      | _ ->
          Error
            "internal CEK error: unexpected frame after statement completion")

type code = expr
type ainput = N.t list
type rinput = unit
type routput = print_effect
type foutput = N.t
type state = { code : expr; state : phase } [@@deriving yojson]

let make code inputs =
  Ok { code; state = EvalExpr (apply_inputs code inputs, [], [ Init ]) }

let step s () =
  match s.state with
  | Return (v, [ Init ]) ->
      let* n = as_nat v in
      Ok (Utils_runtime.Machine.Halt n)
  | _ ->
      let* state, output = step_state s.state in
      Ok (Utils_runtime.Machine.Continue { next = { s with state }; output })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson
  let of_yojson = state_of_yojson
end

let render s =
  let env_from_kont k =
    List.find_map
      (function
        | BinOpLeft (_, _, env)
        | AppFun (_, env)
        | IfExpr (_, _, env)
        | LetFrame (_, env)
        | IfStmtFrame (_, env)
        | WhileFrame (_, _, env) ->
            Some env
        | _ -> None)
      k
    |> Option.value ~default:[]
  in
  let mode, focus, env, k =
    match s.state with
    | EvalExpr (e, env, k) -> ("eval-expr", Code.print e, env, k)
    | EvalStmt (e, env, k) -> ("eval-stmt", Code.print_stmt e, env, k)
    | Return (v, k) -> ("return", value_text v, env_from_kont k, k)
    | Done (env, k) -> ("done", "scope complete", env, k)
  in
  Pcf_shared.View.continuation ~code:(Code.print s.code) ~mode ~focus
    ~bindings:env
    ~frame_text:(fun f -> Yojson.Safe.to_string (frame_to_yojson f))
    k

module Ainput = Pcf_shared.Io.Ainput
module Rinput = Utils_data.Text.Unit
module Routput = Pcf_shared.Io.Routput
module Foutput = Pcf_shared.Io.Foutput
