open Pcf_shared.Syntax
module Code = Pcf_shared.Syntax.Code

let ( let* ) = Result.bind

let eval_binop lhs op rhs =
  match (op, lhs, rhs) with
  | Add, Nat a, Nat b -> Ok (Nat (N.add a b))
  | Sub, Nat a, Nat b when N.compare a b >= 0 -> Ok (Nat (N.sub a b))
  | Sub, Nat _, Nat _ -> Error "operator '-' expects lhs >= rhs over naturals"
  | And, Bool a, Bool b -> Ok (Bool (a && b))
  | Add, _, _ -> Error "operator '+' expects naturals"
  | Sub, _, _ -> Error "operator '-' expects naturals"
  | And, _, _ -> Error "operator '&&' expects booleans"

let eval_unop op e =
  match (op, e) with
  | Inc, Nat n -> Ok (Nat (N.succ n))
  | Dec, Nat n when not (N.is_zero n) -> Ok (Nat (N.pred n))
  | Dec, Nat _ -> Error "operator 'dec' expects a positive natural"
  | Not, Bool b -> Ok (Bool (not b))
  | Inc, _ -> Error "operator 'inc' expects a natural"
  | Dec, _ -> Error "operator 'dec' expects a natural"
  | Not, _ -> Error "operator 'not' expects a boolean"

let rec small_step = function
  | Print n -> Ok (Unit, Some n)
  | BinOp (a, o, b) when not (is_value a) ->
      let* a, out = small_step a in
      Ok (BinOp (a, o, b), out)
  | BinOp (a, o, b) when not (is_value b) ->
      let* b, out = small_step b in
      Ok (BinOp (a, o, b), out)
  | BinOp (a, o, b) ->
      let* e = eval_binop a o b in
      Ok (e, None)
  | UnOp (o, e) when not (is_value e) ->
      let* e, out = small_step e in
      Ok (UnOp (o, e), out)
  | UnOp (o, e) ->
      let* e = eval_unop o e in
      Ok (e, None)
  | App (f, a) when not (is_value f) ->
      let* f, out = small_step f in
      Ok (App (f, a), out)
  | App (f, a) when not (is_value a) ->
      let* a, out = small_step a in
      Ok (App (f, a), out)
  | App (Fun (x, b), a) -> Ok (substitute b x a, None)
  | App ((Rec (f, x, b) as r), a) -> Ok (substitute (substitute b f r) x a, None)
  | App (f, _) ->
      Error ("application expects a function value, found " ^ Code.print f)
  | If (c, t, e) when not (is_value c) ->
      let* c, out = small_step c in
      Ok (If (c, t, e), out)
  | If (Bool true, t, _) -> Ok (t, None)
  | If (Bool false, _, e) -> Ok (e, None)
  | If (c, _, _) ->
      Error ("if expects a boolean condition, found " ^ Code.print c)
  | Var x -> Error ("unbound variable: " ^ x)
  | Block _ -> Error "blocks belong to the expression/statement machine"
  | _ -> Error "cannot step a value"

type code = expr [@@deriving yojson]
type ainput = N.t list
type rinput = unit
type routput = print_effect
type foutput = N.t
type state = { code : code; expr : expr } [@@deriving yojson]

let make code inputs = Ok { code; expr = apply_inputs code inputs }

let step state () =
  if is_value state.expr then
    let* n = expect_nat state.expr in
    Ok (Utils.Machine.Halt n)
  else
    let* expr, output = small_step state.expr in
    Ok (Utils.Machine.Continue { next = { state with expr }; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.state
    [
      Utils.Render.text ~title:"code" (Code.print s.code);
      Utils.Render.text ~title:"current" (Code.print s.expr);
    ]

module Ainput = Pcf_shared.Syntax.Ainput
module Rinput = Utils.Text.Unit
module Routput = Pcf_shared.Syntax.Routput
module Foutput = Pcf_shared.Syntax.Foutput
