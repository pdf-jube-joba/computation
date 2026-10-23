open Sexp
open Utils_runtime.Machine

type env = (string * Sexp.t) list [@@deriving yojson]

type frame =
  | Operator of Sexp.t list * env
  | Argument of Sexp.t * Sexp.t list * Sexp.t list * env
  | Branch of Sexp.t * (Sexp.t * Sexp.t) list * env
[@@deriving yojson]

type control =
  | Eval of Sexp.t * env
  | Return of Sexp.t * env
  | Apply of Sexp.t * Sexp.t list * env
[@@deriving yojson]

type state = { code : Sexp.t; control : control; kont : frame list }
[@@deriving yojson]

module Code = Sexp
module Ainput = Utils_data.Text.Unit
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Sexp

let primitives = [ "atom"; "eq"; "cons"; "car"; "cdr"; "eval" ]
let initial_env = List.map (fun name -> (name, Symbol name)) primitives
let make code () = Ok { code; control = Eval (code, initial_env); kont = [] }

let binder = function
  | Symbol name when name <> "t" -> Ok name
  | _ -> Error "Expected a variable name (not nil or t)"

let parameters value =
  let* values = to_list value in
  let rec loop seen = function
    | [] -> Ok (List.rev seen)
    | value :: rest ->
        let* name = binder value in
        if List.mem name seen then Error ("Duplicate parameter: " ^ name)
        else loop (name :: seen) rest
  in
  loop [] values

let lambda value =
  match value with
  | Cons (Symbol "lambda", Cons (params, Cons (body, Nil))) ->
      let* names = parameters params in
      Ok (names, body)
  | _ -> Error "Expected (lambda (parameters ...) body)"

let labelled value =
  match value with
  | Cons (Symbol "label", Cons (name, Cons (fn, Nil))) ->
      let* name = binder name in
      let* names, body = lambda fn in
      Ok (name, names, body)
  | _ -> Error "Expected (label name (lambda (parameters ...) body))"

let clauses values =
  let rec loop acc = function
    | [] -> Ok (List.rev acc)
    | Cons (test, Cons (body, Nil)) :: rest -> loop ((test, body) :: acc) rest
    | _ -> Error "Each cond clause must be (test body)"
  in
  loop [] values

let branch env kont = function
  | [] -> (Return (Nil, env), kont)
  | (test, body) :: rest -> (Eval (test, env), Branch (body, rest, env) :: kont)

let arguments fn args env kont =
  match args with
  | [] -> (Apply (fn, [], env), kont)
  | arg :: rest -> (Eval (arg, env), Argument (fn, [], rest, env) :: kont)

let truth b = if b then Symbol "t" else Nil

let apply fn args env =
  let return value = Ok (Return (value, env)) in
  let bind names body env =
    if List.length names <> List.length args then
      Error "Wrong number of arguments to lambda"
    else Ok (Eval (body, List.combine names args @ env))
  in
  match (fn, args) with
  | Symbol "atom", [ value ] ->
      return (truth (match value with Cons _ -> false | _ -> true))
  | Symbol "eq", [ left; right ] ->
      return
        (truth
           (match (left, right) with
           | Nil, Nil -> true
           | Symbol x, Symbol y -> x = y
           | _ -> false))
  | Symbol "cons", [ head; tail ] -> return (Cons (head, tail))
  | Symbol "car", [ Cons (head, _) ] -> return head
  | Symbol "cdr", [ Cons (_, tail) ] -> return tail
  | Symbol ("car" | "cdr"), [ Nil ] -> return Nil
  | Symbol ("car" | "cdr"), [ _ ] -> Error "car/cdr expects a pair or nil"
  | Symbol "eval", [ expression ] -> Ok (Eval (expression, env))
  | Symbol name, _ when List.mem name primitives ->
      Error ("Wrong number of arguments to " ^ name)
  | Cons (Symbol "lambda", _), _ ->
      let* names, body = lambda fn in
      bind names body env
  | Cons (Symbol "label", _), _ ->
      let* name, names, body = labelled fn in
      bind names body ((name, fn) :: env)
  | _ -> Error ("Not a function: " ^ Sexp.print fn)

(* Every call uses the caller's environment. Continuation frames retain that
   environment so bindings from a completed call cannot leak into its siblings. *)
let transition s =
  let return value env = Ok (Return (value, env), s.kont) in
  match s.control with
  | Eval (((Nil | Symbol "t") as value), env) -> return value env
  | Eval (Symbol name, env) -> (
      match List.assoc_opt name env with
      | Some value -> return value env
      | None -> Error ("Unbound symbol: " ^ name))
  | Eval ((Cons (operator, operands) as expression), env) -> (
      let* args = to_list operands in
      match (operator, args) with
      | Symbol "quote", [ value ] -> return value env
      | Symbol "quote", _ -> Error "quote expects one argument"
      | Symbol "lambda", _ ->
          let* _ = lambda expression in
          return expression env
      | Symbol "label", _ ->
          let* _ = labelled expression in
          return expression env
      | Symbol "cond", _ ->
          let* choices = clauses args in
          Ok (branch env s.kont choices)
      | _ -> Ok (Eval (operator, env), Operator (args, env) :: s.kont))
  | Apply (fn, args, env) ->
      let* control = apply fn args env in
      Ok (control, s.kont)
  | Return (value, _) -> (
      match s.kont with
      | Operator (args, env) :: kont -> Ok (arguments value args env kont)
      | Argument (fn, done_rev, rest, env) :: kont -> (
          match rest with
          | [] -> Ok (Apply (fn, List.rev (value :: done_rev), env), kont)
          | arg :: rest ->
              Ok
                ( Eval (arg, env),
                  Argument (fn, value :: done_rev, rest, env) :: kont ))
      | Branch (body, rest, env) :: kont ->
          if value = Nil then Ok (branch env kont rest)
          else Ok (Eval (body, env), kont)
      | [] -> Error "Evaluation already finished")

let step s () =
  match (s.control, s.kont) with
  | Return (value, _), [] -> Ok (Halt value)
  | _ ->
      let* control, kont = transition s in
      Ok (Continue { next = { s with control; kont }; output = () })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson
  let of_yojson = state_of_yojson
end

let render s =
  let mode, focus, env =
    match s.control with
    | Eval (expr, env) -> ("eval", Sexp.print expr, env)
    | Return (value, env) -> ("return", Sexp.print value, env)
    | Apply (fn, args, env) ->
        ("apply", Sexp.print (Sexp.list (fn :: args)), env)
  in
  let open Utils_runtime.Render in
  [
    text ~title:"code" (Code.print s.code);
    text_table ~title:"state" ~columns:[ "field"; "value" ]
      [ [ "mode"; mode ]; [ "focus"; focus ] ];
    assoc_table ~title:"env" ~columns:[ "name"; "value" ] Sexp.print env;
    indexed_table ~title:"kont" ~columns:[ "depth"; "frame" ]
      (fun frame -> Yojson.Safe.to_string (frame_to_yojson frame))
      s.kont;
  ]
