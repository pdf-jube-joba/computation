open Syntax
module Var_id = Utils_names.Id.Make ()

type environment = {
  names : (Utils_names.Identifier.t * Var_id.t) list;
  values : (Var_id.t * T.sign) list;
}
[@@deriving yojson]

let env_of_named entries =
  let supply = ref Var_id.Supply.empty in
  let names =
    List.map (fun (name, _) -> (name, Var_id.Supply.take supply)) entries
  in
  let values =
    List.map2 (fun (_, id) (_, value) -> (id, value)) names entries
  in
  { names; values }

let env_create vars =
  env_of_named
    (List.map (fun name -> (name, None)) (List.sort_uniq String.compare vars))

let env_named env =
  List.map
    (fun (name, id) ->
      (name, Option.value ~default:None (List.assoc_opt id env.values)))
    env.names

let variable env name =
  Utils_base.Assoc.lookup name env.names ("Unknown variable: " ^ name)

let validate_environment vars env =
  let expected = env_create vars in
  if env.names <> expected.names then
    failwith "Inconsistent variable IDs in environment";
  let ids = List.map fst env.values in
  let used = Var_id.Set.of_list ids in
  if
    List.length ids <> Var_id.Set.cardinal used
    || not (Var_id.Set.equal used (Var_id.Set.of_list (List.map snd env.names)))
  then failwith "Inconsistent environment values"
