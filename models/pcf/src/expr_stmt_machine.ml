open Syntax

let ( let* ) = Result.bind

type value = Cek_machine.value [@@deriving yojson]
type env = Cek_machine.env [@@deriving yojson]

type frame =
  | BinOpLeft of binop * expr * env
  | BinOpRight of value * binop
  | Unary of unop
  | AppFun of expr * env
  | AppArg of value
  | IfExpr of expr * expr * env
  | LetFrame of Utils.Identifier.t * env
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
      | Nat n -> Ok (Return (Cek_machine.VNat n, k), None)
      | Bool b -> Ok (Return (Cek_machine.VBool b, k), None)
      | Unit -> Ok (Return (Cek_machine.VUnit, k), None)
      | Var x ->
          let* v = Cek_machine.lookup_env env x in
          Ok (Return (v, k), None)
      | Fun (x, b) -> Ok (Return (Cek_machine.VFun (x, b, env), k), None)
      | Rec (f, x, b) -> Ok (Return (Cek_machine.VRec (f, x, b, env), k), None)
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
          let* v = Cek_machine.eval_binop a o v in
          Ok (Return (v, k), None)
      | Unary o ->
          let* v = Cek_machine.eval_unop o v in
          Ok (Return (v, k), None)
      | AppFun (a, env) -> Ok (EvalExpr (a, env, AppArg v :: k), None)
      | AppArg f ->
          let* b, env = Cek_machine.apply f v in
          Ok (EvalExpr (b, env, k), None)
      | IfExpr (t, e, env) ->
          let* b = Cek_machine.boolean "if" v in
          Ok (EvalExpr ((if b then t else e), env, k), None)
      | LetFrame (x, env) -> Ok (Done ((x, v) :: env, k), None)
      | IfStmtFrame (s, env) ->
          let* b = Cek_machine.boolean "if" v in
          Ok ((if b then EvalStmt (s, env, k) else Done (env, k)), None)
      | WhileFrame (c, s, env) ->
          let* b = Cek_machine.boolean "while" v in
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

let parse_code = Syntax.parse_stmt_code
let parse_ainput = Syntax.parse_ainput
let parse_rinput = Utils.Machine.empty_input

let make code inputs =
  Ok { code; state = EvalExpr (apply_inputs code inputs, [], [ Init ]) }

let step s () =
  match s.state with
  | Return (v, [ Init ]) ->
      let* n = Cek_machine.as_nat v in
      Ok (Utils.Machine.Halt n)
  | _ ->
      let* state, output = step_state s.state in
      Ok (Utils.Machine.Continue { next = { s with state }; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

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

let render s =
  let mode, focus, env, k =
    match s.state with
    | EvalExpr (e, env, k) -> ("eval-expr", print e, env, k)
    | EvalStmt (e, env, k) -> ("eval-stmt", print_stmt e, env, k)
    | Return (v, k) -> ("return", Cek_machine.value_text v, env_from_kont k, k)
    | Done (env, k) -> ("done", "scope complete", env, k)
  in
  let table = Cek_machine.table in
  Utils.Render.state
    [
      Utils.Render.text ~title:"code" (print s.code);
      table "state" [ "field"; "value" ]
        [ [ "mode"; mode ]; [ "focus"; focus ] ];
      table "env" [ "name"; "value" ]
        (List.rev_map (fun (x, v) -> [ x; Cek_machine.value_text v ]) env);
      table "kont" [ "depth"; "frame" ]
        (List.mapi
           (fun i f ->
             [ string_of_int i; Yojson.Safe.to_string (frame_to_yojson f) ])
           k);
    ]

let routput_json = Syntax.routput_json
let foutput_json = Syntax.foutput_json
