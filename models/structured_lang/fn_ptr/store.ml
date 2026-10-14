open Runtime_types

let bindings frame_id params values =
  List.fold_left
    (fun (env, store) (slot, (p, v)) ->
      let loc = { frame_id; slot } in
      (Utils_base.Assoc.set p loc env, (loc, v) :: store))
    ([], [])
    (List.mapi (fun i pv -> (i, pv)) (List.combine params values))

let store_lookup s loc =
  let rec loop = function
    | [] -> failwith ("unknown location: " ^ Runtime_value.location_text loc)
    | f :: fs -> (
        match List.assoc_opt loc f.values with Some v -> v | None -> loop fs)
  in
  loop s.store

let store_assign s loc value =
  let rec loop = function
    | [] -> failwith ("unknown location: " ^ Runtime_value.location_text loc)
    | f :: fs ->
        if List.mem_assoc loc f.values then
          { f with values = Utils_base.Assoc.set loc value f.values } :: fs
        else f :: loop fs
  in
  { s with store = loop s.store }
