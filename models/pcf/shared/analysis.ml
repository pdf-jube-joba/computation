open Syntax
module Names = Utils_names.Identifier.Set

let rec free_vars = function
  | Nat _ | Bool _ | Unit | Print _ -> Names.empty
  | Var x -> Names.singleton x
  | BinOp (a, _, b) | App (a, b) -> Names.union (free_vars a) (free_vars b)
  | UnOp (_, e) -> free_vars e
  | Fun (x, e) -> Names.remove x (free_vars e)
  | Rec (f, x, e) -> Names.remove f (Names.remove x (free_vars e))
  | If (c, t, e) ->
      Names.union (free_vars c) (Names.union (free_vars t) (free_vars e))
  | Block _ ->
      invalid_arg "substitution applies to the direct expression language"
