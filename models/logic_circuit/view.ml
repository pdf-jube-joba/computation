open Circuit

let as_graph_group = function
  | Gate _ as c ->
      {
        verts = [ (External (Utils_names.Identifier.make_exn (kind c)), c) ];
        edges = [];
        inpins_map = List.map (fun p -> (p, p)) (get_inpins c);
        otpins_map = List.map (fun p -> (p, p)) (get_otpins c);
      }
  | Mix m ->
      {
        verts = List.map (fun (name, c) -> (Internal name, c)) m.verts;
        edges = List.map (fun (a, b) -> (internal a, internal b)) m.edges;
        inpins_map =
          List.map
            (fun p -> (map_internal m.kind m.inpin_maps p, internal p))
            (usable_mix_inpins m);
        otpins_map =
          List.map
            (fun p -> (map_internal m.kind m.otpin_maps p, internal p))
            (usable_mix_otpins m);
      }
  | Iter i ->
      let edges =
        List.concat
          (List.mapi
             (fun n _ ->
               (if n < List.length i.used - 1 then
                  List.map
                    (fun (a, b) -> (instance n a, instance (n + 1) b))
                    i.next_edges
                else [])
               @
               if n > 0 then
                 List.map
                   (fun (a, b) -> (instance n a, instance (n - 1) b))
                   i.prev_edges
               else [])
             i.used)
      in
      {
        verts = List.mapi (fun n c -> (Instance (id n), c)) i.used;
        edges;
        inpins_map = List.map (fun p -> (p, p)) (usable_iter_inpins i);
        otpins_map = List.map (fun p -> (p, p)) (usable_iter_otpins i);
      }

let render circuit =
  let graph = as_graph_group circuit in
  let names = ref Vertex_names.empty in
  List.iter
    (fun (owner, _) ->
      match owner with
      | Internal name | External name ->
          names := Vertex_names.reserve name !names
      | Instance _ -> ())
    graph.verts;
  let display = function
    | Instance id ->
        let name, next =
          Vertex_names.assign
            ~hint:(Utils_names.Identifier.make_exn "instance")
            id !names
        in
        names := next;
        Utils_names.Identifier.to_string name
    | owner -> owner_text owner
  in
  let inputs =
    List.map
      (fun ((owner, p), _) ->
        [ display owner; Utils_names.Identifier.to_string p ])
      graph.inpins_map
  in
  let outputs =
    List.map
      (fun ((owner, p), (vert, pin)) ->
        let value =
          match List.assoc_opt vert graph.verts with
          | None -> "?"
          | Some c -> (
              match
                List.find_opt (fun ((_, p), _) -> p = pin) (get_otputs c)
              with
              | Some (_, b) -> Signal_text.print_bool b
              | None -> "?")
        in
        [ display owner; Utils_names.Identifier.to_string p; value ])
      graph.otpins_map
  in
  let verts =
    List.concat_map
      (fun (owner, c) ->
        List.map
          (fun ((_, p), v) ->
            [
              display owner;
              Utils_names.Identifier.to_string p;
              Signal_text.print_bool v;
            ])
          (get_otputs c))
      graph.verts
  in
  Utils_runtime.Render.state
    [
      Utils_runtime.Render.text ~title:"kind" (kind circuit);
      Utils_runtime.Render.text_table ~title:"inputs" ~columns:[ "name"; "pin" ]
        inputs;
      Utils_runtime.Render.text_table ~title:"outputs"
        ~columns:[ "name"; "pin"; "value" ] outputs;
      Utils_runtime.Render.text_table ~title:"verts"
        ~columns:[ "vert"; "pin"; "value" ] verts;
    ]
