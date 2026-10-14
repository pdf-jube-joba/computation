open Runtime_types
open Runtime_print
module Code = Parser.Code

let render s =
  let open Utils_runtime.Render in
  let cell_rows to_int cells =
    List.map
      (fun (id, c) ->
        [ string_of_int (to_int id); Code.type_text c.ty; value_text c.value ])
      (Utils_base.Assoc.sorted cells)
  in
  state
    [
      text_table ~title:"status" ~columns:[ "field"; "value" ]
        [
          [ "function"; Utils_names.Identifier.to_string s.current_function ];
          [ "exec"; exec_text s.exec ];
          [
            "next_local_id";
            string_of_int (Local_id.Supply.to_int s.next_local_id);
          ];
          [
            "next_heap_id"; string_of_int (Heap_id.Supply.to_int s.next_heap_id);
          ];
        ];
      text_table ~title:"env"
        ~columns:[ "scope"; "name"; "local" ]
        (List.concat
           (List.mapi
              (fun i scope ->
                List.map
                  (fun (n, id) ->
                    [
                      string_of_int i;
                      Utils_names.Identifier.to_string n;
                      string_of_int (Local_id.to_int id);
                    ])
                  (Utils_base.Assoc.sorted scope))
              s.env));
      text_table ~title:"statics"
        ~columns:[ "name"; "type"; "value" ]
        (List.map
           (fun (n, c) ->
             [
               Utils_names.Identifier.to_string n;
               Code.type_text c.ty;
               value_text c.value;
             ])
           (Utils_base.Assoc.sorted s.store.statics));
      text_table ~title:"heap"
        ~columns:[ "heap"; "type"; "value" ]
        (cell_rows Heap_id.to_int s.store.heap);
      container ~title:"locals"
        (List.mapi
           (fun i scopes ->
             container
               ~title:("fn frame " ^ string_of_int i)
               (List.mapi
                  (fun j cells ->
                    text_table
                      ~title:("scope " ^ string_of_int j)
                      ~columns:[ "local"; "type"; "value" ]
                      (cell_rows Local_id.to_int cells))
                  scopes))
           s.store.locals);
      indexed_table ~title:"inner_k" ~columns:[ "depth"; "frame" ] inner_text
        s.inner_k;
      text_table ~title:"fn_k"
        ~columns:[ "depth"; "function"; "rets"; "resume" ]
        (List.mapi
           (fun i f ->
             [
               string_of_int i;
               Utils_names.Identifier.to_string f.function_name;
               string_of_int (List.length f.ret_locs);
               string_of_int (List.length f.inner_k);
             ])
           s.fn_k);
    ]
