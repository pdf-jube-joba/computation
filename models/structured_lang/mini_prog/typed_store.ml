open Runtime_types

let rec type_match v ty =
  match (v, ty) with
  | Number_value _, Num -> true
  | Number_value n, U8 -> N.compare n (N.of_int 255) <= 0
  | Bool_value _, Bool | Unit_v, Unit | Ptr_value _, Ptr | FnPtr _, Fn -> true
  | Aggregate vs, Product ts ->
      List.length vs = List.length ts && List.for_all2 type_match vs ts
  | Aggregate vs, Array (t, n) ->
      List.length vs = n && List.for_all (fun v -> type_match v t) vs
  | Sum_value (tag, v), Sum ts -> (
      match List.nth_opt ts tag with Some t -> type_match v t | None -> false)
  | _ -> false

let rec default_value = function
  | Num | U8 -> Number_value N.zero
  | Bool -> Bool_value false
  | Unit -> Unit_v
  | Ptr -> Ptr_value None
  | Fn -> FnPtr None
  | Product ts -> Aggregate (List.map default_value ts)
  | Array (t, n) -> Aggregate (List.init n (fun _ -> default_value t))
  | Sum [] -> failwith "sum type must have at least one variant"
  | Sum (t :: _) -> Sum_value (0, default_value t)

let rec project_cell ty value = function
  | [] -> (ty, value)
  | Field_proj i :: ps -> (
      match (ty, value) with
      | Product ts, Aggregate vs ->
          project_cell
            (Utils_base.List_ext.nth ts i "projection out of bounds")
            (Utils_base.List_ext.nth vs i "projection out of bounds")
            ps
      | _ -> failwith "invalid place projection")
  | Index_proj i :: ps -> (
      match (ty, value) with
      | Array (t, n), Aggregate vs ->
          if i >= n then failwith "projection out of bounds";
          project_cell t
            (Utils_base.List_ext.nth vs i "projection out of bounds")
            ps
      | _ -> failwith "invalid place projection")
  | Tag_proj i :: ps -> (
      match (ty, value) with
      | Sum ts, Sum_value (tag, v) ->
          if i <> tag then failwith "sum tag mismatch";
          project_cell
            (Utils_base.List_ext.nth ts i "sum tag out of bounds")
            v ps
      | _ -> failwith "invalid place projection")

let rec update_projection ty current ps value =
  match ps with
  | [] ->
      if type_match value ty then value
      else failwith "assigned value does not match location type"
  | Field_proj i :: ps -> (
      match (ty, current) with
      | Product ts, Aggregate vs ->
          let t = Utils_base.List_ext.nth ts i "field projection out of bounds"
          and old =
            Utils_base.List_ext.nth vs i "field projection out of bounds"
          in
          Aggregate
            (Utils_base.List_ext.replace vs i
               (update_projection t old ps value)
               "field projection out of bounds")
      | _ -> failwith "invalid assignment target")
  | Index_proj i :: ps -> (
      match (ty, current) with
      | Array (t, n), Aggregate vs ->
          if i >= n then failwith "array index out of bounds";
          let old = Utils_base.List_ext.nth vs i "array index out of bounds" in
          Aggregate
            (Utils_base.List_ext.replace vs i
               (update_projection t old ps value)
               "array index out of bounds")
      | _ -> failwith "invalid assignment target")
  | Tag_proj i :: ps -> (
      match (ty, current) with
      | Sum ts, Sum_value (tag, v) ->
          if i <> tag then failwith "sum tag mismatch";
          let t = Utils_base.List_ext.nth ts i "sum tag out of bounds" in
          Sum_value (tag, update_projection t v ps value)
      | _ -> failwith "invalid assignment target")

let get_cell s = function
  | Heap id ->
      Utils_base.Assoc.lookup id s.store.heap
        ("unknown heap location: " ^ string_of_int (Heap_id.to_int id))
  | Static_key name ->
      Utils_base.Assoc.lookup name s.store.statics
        ("unknown static location: " ^ Utils_names.Identifier.to_string name)
  | Local_key id ->
      let rec functions = function
        | [] ->
            failwith
              ("unknown local location: " ^ string_of_int (Local_id.to_int id))
        | scopes :: fs ->
            let rec find = function
              | [] -> functions fs
              | cells :: scopes -> (
                  match List.assoc_opt id cells with
                  | Some c -> c
                  | None -> find scopes)
            in
            find scopes
      in
      functions s.store.locals

let set_cell s key cell =
  ignore (get_cell s key);
  let store =
    match key with
    | Heap id ->
        { s.store with heap = Utils_base.Assoc.set id cell s.store.heap }
    | Static_key name ->
        {
          s.store with
          statics = Utils_base.Assoc.set name cell s.store.statics;
        }
    | Local_key id ->
        let rec funcs = function
          | [] -> assert false
          | scopes :: fs -> (
              let rec scope_loop = function
                | [] -> None
                | cells :: scopes ->
                    if List.mem_assoc id cells then
                      Some (Utils_base.Assoc.set id cell cells :: scopes)
                    else
                      Option.map
                        (fun scopes -> cells :: scopes)
                        (scope_loop scopes)
              in
              match scope_loop scopes with
              | Some scopes -> scopes :: fs
              | None -> scopes :: funcs fs)
        in
        { s.store with locals = funcs s.store.locals }
  in
  { s with store }

let lookup_location s loc =
  let cell = get_cell s loc.key in
  project_cell cell.ty cell.value loc.projections

let store_assign s loc value =
  let cell = get_cell s loc.key in
  set_cell s loc.key
    {
      cell with
      value = update_projection cell.ty cell.value loc.projections value;
    }
