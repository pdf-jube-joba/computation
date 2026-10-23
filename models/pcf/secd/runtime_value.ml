open Pcf_shared.Syntax
open Types

include Pcf_shared.Primitive.Make (struct
  type t = value

  let view = function
    | VNat n -> Pcf_shared.Primitive.Nat n
    | VBool b -> Pcf_shared.Primitive.Bool b
    | _ -> Pcf_shared.Primitive.Other

  let natural n = VNat n
  let boolean b = VBool b
end)

let as_nat = function
  | VNat n -> Ok n
  | _ -> Error "final value is not a natural number"

let value_text = function
  | VNat n -> N.to_string n
  | VBool b -> if b then "\\true" else "\\false"
  | VUnit -> "\\unit"
  | VFun (x, c, _) ->
      "closure " ^ Utils_names.Identifier.to_string x ^ " " ^ Code.print c
  | VRec (f, x, c, _) ->
      "recursive closure "
      ^ Utils_names.Identifier.to_string f
      ^ " "
      ^ Utils_names.Identifier.to_string x
      ^ " " ^ Code.print c

let lookup_env (env : env) x =
  match List.assoc_opt x env with
  | Some v -> Ok v
  | None -> Error ("unbound variable: " ^ Utils_names.Identifier.to_string x)
