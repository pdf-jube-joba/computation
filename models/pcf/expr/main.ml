open Pcf_shared.Syntax
module Code = Pcf_shared.Code

let ( let* ) = Result.bind

include Pcf_shared.Primitive.Make (struct
  type t = expr

  let view = function
    | Nat n -> Pcf_shared.Primitive.Nat n
    | Bool b -> Pcf_shared.Primitive.Bool b
    | _ -> Pcf_shared.Primitive.Other

  let natural n = Nat n
  let boolean b = Bool b
end)

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
  | App (Fun (x, b), a) -> Ok (Pcf_shared.Substitution.substitute b x a, None)
  | App ((Rec (f, x, b) as r), a) ->
      Ok
        ( Pcf_shared.Substitution.substitute
            (Pcf_shared.Substitution.substitute b f r)
            x a,
          None )
  | App (f, _) ->
      Error ("application expects a function value, found " ^ Code.print f)
  | If (c, t, e) when not (is_value c) ->
      let* c, out = small_step c in
      Ok (If (c, t, e), out)
  | If (Bool true, t, _) -> Ok (t, None)
  | If (Bool false, _, e) -> Ok (e, None)
  | If (c, _, _) ->
      Error ("if expects a boolean condition, found " ^ Code.print c)
  | Var x -> Error ("unbound variable: " ^ Utils_names.Identifier.to_string x)
  | Block _ -> Error "blocks belong to the expression/statement machine"
  | _ -> Error "cannot step a value"

type code = expr [@@deriving yojson]
type ainput = N.t list
type rinput = unit
type routput = print_effect
type foutput = N.t
type state = { code : code; expr : expr } [@@deriving yojson]

let make code inputs =
  let* () = validate_direct code in
  Ok { code; expr = apply_inputs code inputs }

let step state () =
  if is_value state.expr then
    let* n = expect_nat state.expr in
    Ok (Utils_runtime.Machine.Halt n)
  else
    let* expr, output = small_step state.expr in
    Ok (Utils_runtime.Machine.Continue { next = { state with expr }; output })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson
  let of_yojson = state_of_yojson
end

let render s =
  [
    Utils_runtime.Render.text ~title:"code" (Code.print s.code);
    Utils_runtime.Render.text ~title:"current" (Code.print s.expr);
  ]

module Ainput = Pcf_shared.Io.Ainput
module Rinput = Utils_data.Text.Unit
module Routput = Pcf_shared.Io.Routput
module Foutput = Pcf_shared.Io.Foutput
