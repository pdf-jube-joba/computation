include Lambda_continuations_shared.Syntax

let rec reduce = function
  | Var _ | Lam _ | Zero -> None
  | App (f, a) -> (
      match (is_value f, is_value a) with
      | Some (Fun (x, b)), Some _ -> Some (subst b x a)
      | Some (Num _), Some _ -> None
      | Some _, None -> Option.map (fun a -> App (f, a)) (reduce a)
      | None, _ -> Option.map (fun f -> App (f, a)) (reduce f))
  | Succ e ->
      if Option.is_none (is_value e) then
        Option.map (fun e -> Succ e) (reduce e)
      else None
  | Pred e -> (
      match is_value e with
      | Some (Num n) -> Some (num_to_exp (N.pred n))
      | Some (Fun _) -> None
      | None -> Option.map (fun e -> Pred e) (reduce e))
  | IfZ (c, t, f) -> (
      match is_value c with
      | Some (Num n) -> Some (if N.is_zero n then t else f)
      | Some (Fun _) -> None
      | None -> Option.map (fun c -> IfZ (c, t, f)) (reduce c))
  | Let (x, e, b) -> Some (App (Lam (x, b), e))
  | Rec (f, x, b) as r -> Some (Lam (x, subst b f r))
  | Abort _ | Control _ -> None

type code = term
type ainput = N.t list
type rinput = unit
type routput = unit
type foutput = value
type state = { code : term; current : term } [@@deriving yojson]

let make code inputs =
  Ok
    {
      code;
      current = List.fold_left (fun f n -> App (f, num_to_exp n)) code inputs;
    }

let step s () =
  match reduce s.current with
  | Some current ->
      Ok
        (Utils_runtime.Machine.Continue
           { next = { s with current }; output = () })
  | None -> (
      match is_value s.current with
      | Some value -> Ok (Utils_runtime.Machine.Halt value)
      | None -> Error ("stuck term: " ^ Code.pretty s.current))

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson
  let of_yojson = state_of_yojson
end

let render s =
  [
    Utils_runtime.Render.text ~title:"code" (Code.pretty s.code);
    Utils_runtime.Render.text ~title:"current" (Code.pretty s.current);
  ]

module Ainput = Utils_data.Text.Natural_list
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Lambda_continuations_shared.Syntax.Value
