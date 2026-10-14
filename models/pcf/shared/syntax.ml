module N = Utils_base.Number

type binop = Add | Sub | And [@@deriving yojson]
type unop = Inc | Dec | Not [@@deriving yojson]

(* Blocks are the statement-language extension; the expression parser rejects them. *)
type expr =
  | Nat of N.t
  | Bool of bool
  | Unit
  | Print of N.t
  | Var of Utils_names.Identifier.t
  | BinOp of expr * binop * expr
  | UnOp of unop * expr
  | Fun of Utils_names.Identifier.t * expr
  | Rec of Utils_names.Identifier.t * Utils_names.Identifier.t * expr
  | App of expr * expr
  | If of expr * expr * expr
  | Block of stmt * expr

and stmt =
  | Skip
  | PrintStmt of N.t
  | Seq of stmt * stmt
  | Let of Utils_names.Identifier.t * expr
  | IfStmt of expr * stmt
  | While of expr * stmt
[@@deriving yojson]

type print_effect = N.t option [@@deriving yojson]

let is_value = function
  | Nat _ | Bool _ | Unit | Fun _ | Rec _ -> true
  | _ -> false

let expect_nat = function
  | Nat n -> Ok n
  | _ -> Error "final value is not a natural number"

let apply_inputs e inputs = List.fold_left (fun e n -> App (e, Nat n)) e inputs

(** The base language does not contain statement blocks, including in function
    bodies. *)
let rec validate_direct = function
  | Block _ -> Error "blocks belong to the expression/statement machine"
  | BinOp (a, _, b) | App (a, b) ->
      Result.bind (validate_direct a) (fun () -> validate_direct b)
  | UnOp (_, e) | Fun (_, e) | Rec (_, _, e) -> validate_direct e
  | If (c, t, e) ->
      Result.bind (validate_direct c) (fun () ->
          Result.bind (validate_direct t) (fun () -> validate_direct e))
  | Nat _ | Bool _ | Unit | Print _ | Var _ -> Ok ()
