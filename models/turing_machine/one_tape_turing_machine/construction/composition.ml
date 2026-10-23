open One_tape_turing_machine.Main
open Builder
module Identifier = Utils_names.Identifier

type 'sign graph = {
  name : string;
  init_state : string;
  assign_vertex_to_builder : 'sign t list;
  assign_edge_to_state : ((int * int) * string) list;
  acceptable : string list list;
}

let builder_composition g =
  Utils_base.Error.protect (fun () ->
      let vertices = Array.of_list g.assign_vertex_to_builder in
      if Array.length vertices <> List.length g.acceptable then
        failwith "Length of vertices and acceptable states differ";
      if Array.length vertices = 0 then failwith "No vertices";
      List.iter
        (fun ((i, j), _) ->
          if
            i < 0 || j < 0
            || i >= Array.length vertices
            || j >= Array.length vertices
          then failwith "Edge index out of bounds")
        g.assign_edge_to_state;
      let supply = ref State_id.Supply.empty in
      let names = ref Names.empty in
      let allocate hint =
        let hint = unwrap (State.parse hint) in
        let id = State_id.Supply.take supply in
        let _, next = Names.assign ~hint id !names in
        names := next;
        id
      in
      let external_states = ref Identifier.Map.empty in
      let external_state hint =
        let hint = unwrap (State.parse hint) in
        match Identifier.Map.find_opt hint !external_states with
        | Some id -> id
        | None ->
            let id = allocate (Utils_names.Identifier.to_string hint) in
            external_states := Identifier.Map.add hint id !external_states;
            id
      in
      (* Reserve external names before assigning printable hints to copied states.
         Each vertex owns a separate old-state -> new-ID mapping. *)
      let init_state = external_state g.init_state in
      let accepted_state = List.map external_state (List.concat g.acceptable) in
      let copies = Array.make (Array.length vertices) Identifier.Map.empty in
      let state i hint =
        let hint = unwrap (State.parse hint) in
        match Identifier.Map.find_opt hint copies.(i) with
        | Some id -> id
        | None ->
            let id = allocate (Utils_names.Identifier.to_string hint) in
            copies.(i) <- Identifier.Map.add hint id copies.(i);
            id
      in
      let name id =
        Utils_names.Identifier.to_string (Option.get (Names.find id !names))
      in
      let signs =
        List.sort_uniq compare
          (List.concat_map
             (fun b ->
               List.concat_map (fun e -> [ e.key_sign; e.value_sign ]) b.code)
             g.assign_vertex_to_builder)
      in
      let constants a b =
        List.map (fun s -> ((s, a), (s, b, Constant))) signs
      in
      let initial = constants init_state (state 0 vertices.(0).init_state) in
      let vertex_code =
        List.concat
          (List.mapi
             (fun i b ->
               List.map
                 (fun e ->
                   let source = state i e.key_state in
                   let target = state i e.value_state in
                   ((e.key_sign, source), (e.value_sign, target, e.direction)))
                 b.code)
             g.assign_vertex_to_builder)
      in
      let edge_code =
        List.concat_map
          (fun ((i, j), hint) ->
            constants (state i hint) (state j vertices.(j).init_state))
          g.assign_edge_to_state
      in
      let accepted_code =
        List.concat
          (List.mapi
             (fun i states ->
               List.concat_map
                 (fun hint -> constants (state i hint) (external_state hint))
                 states)
             g.acceptable)
      in
      let code = initial @ vertex_code @ edge_code @ accepted_code in
      {
        Builder.name = g.name;
        init_state = name init_state;
        accepted_state = List.map name accepted_state;
        code =
          List.map
            (fun ((sign, source), (next_sign, target, direction)) ->
              Builder.of_entry
                ((sign, name source), (next_sign, name target, direction)))
            code;
      })
