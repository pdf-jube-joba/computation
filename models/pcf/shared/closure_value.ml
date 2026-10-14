open Syntax

let ( let* ) = Result.bind

type value =
  | VNat of N.t
  | VBool of bool
  | VUnit
  | VFun of Utils_names.Identifier.t * expr * env
  | VRec of Utils_names.Identifier.t * Utils_names.Identifier.t * expr * env

and env = (Utils_names.Identifier.t * value) list [@@deriving yojson]

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
  | e -> Error ("expected value expression, found " ^ Code.print e)

include Primitive.Make (struct
  type t = value

  let view = function
    | VNat n -> Primitive.Nat n
    | VBool b -> Primitive.Bool b
    | _ -> Primitive.Other

  let natural n = VNat n
  let boolean b = VBool b
end)

let as_nat = function
  | VNat n -> Ok n
  | _ -> Error "final value is not a natural number"

let value_text v = Code.print (value_to_expr v)

let apply fun_value arg =
  match fun_value with
  | VFun (x, b, env) -> Ok (b, (x, arg) :: env)
  | VRec (f, x, b, env) as r -> Ok (b, (x, arg) :: (f, r) :: env)
  | v -> Error ("application expects a closure, found " ^ value_text v)

let boolean context = function
  | VBool b -> Ok b
  | v -> Error (context ^ " expects a boolean condition, found " ^ value_text v)
