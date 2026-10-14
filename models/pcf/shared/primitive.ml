open Syntax

type scalar = Nat of N.t | Bool of bool | Other

module Make (Value : sig
  type t

  val view : t -> scalar
  val natural : N.t -> t
  val boolean : bool -> t
end) =
struct
  let eval_binop lhs op rhs =
    match (op, Value.view lhs, Value.view rhs) with
    | Add, Nat a, Nat b -> Ok (Value.natural (N.add a b))
    | Sub, Nat a, Nat b when N.compare a b >= 0 ->
        Ok (Value.natural (N.sub a b))
    | Sub, Nat _, Nat _ -> Error "operator '-' expects lhs >= rhs over naturals"
    | And, Bool a, Bool b -> Ok (Value.boolean (a && b))
    | Add, _, _ -> Error "operator '+' expects naturals"
    | Sub, _, _ -> Error "operator '-' expects naturals"
    | And, _, _ -> Error "operator '&&' expects booleans"

  let eval_unop op e =
    match (op, Value.view e) with
    | Inc, Nat n -> Ok (Value.natural (N.succ n))
    | Dec, Nat n when not (N.is_zero n) -> Ok (Value.natural (N.pred n))
    | Dec, Nat _ -> Error "operator 'dec' expects a positive natural"
    | Not, Bool b -> Ok (Value.boolean (not b))
    | Inc, _ -> Error "operator 'inc' expects a natural"
    | Dec, _ -> Error "operator 'dec' expects a natural"
    | Not, _ -> Error "operator 'not' expects a boolean"
end
