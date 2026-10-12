open Main
open Builder

type 'sign graph = {
  name : string;
  init_state : string;
  assign_vertex_to_builder : 'sign t list;
  assign_edge_to_state : ((int * int) * string) list;
  acceptable : string list list;
}

let builder_composition g =
  Utils.Lexer.protect (fun () ->
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
      let name i state =
        Main.unwrap
          (State.parse (Printf.sprintf "v%d_%s_%s" i vertices.(i).name state))
      in
      let signs =
        List.sort_uniq compare
          (List.concat_map
             (fun b ->
               List.concat_map (fun e -> [ e.key_sign; e.value_sign ]) b.code)
             g.assign_vertex_to_builder)
      in
      let constants a b =
        List.map
          (fun s ->
            {
              key_sign = s;
              key_state = a;
              value_sign = s;
              value_state = b;
              direction = Constant;
            })
          signs
      in
      let code =
        constants g.init_state (name 0 vertices.(0).init_state)
        @ List.concat
            (List.mapi
               (fun i b ->
                 List.map
                   (fun e ->
                     {
                       e with
                       key_state = name i e.key_state;
                       value_state = name i e.value_state;
                     })
                   b.code)
               g.assign_vertex_to_builder)
        @ List.concat_map
            (fun ((i, j), state) ->
              constants (name i state) (name j vertices.(j).init_state))
            g.assign_edge_to_state
        @ List.concat
            (List.mapi
               (fun i states ->
                 List.concat_map
                   (fun state -> constants (name i state) state)
                   states)
               g.acceptable)
      in
      {
        Builder.name = g.name;
        init_state = g.init_state;
        accepted_state = List.concat g.acceptable;
        code;
      })
