open Runtime_types
open Store

let render s =
  let open Utils_runtime.Render in
  [
    text_table ~title:"status" ~columns:[ "field"; "value" ]
      [
        [ "control"; (if s.current_stmt = None then "next" else "eval") ];
        [ "function"; Utils_names.Identifier.to_string s.current_function ];
        [
          "current";
          Option.fold ~none:"<none>" ~some:Code.stmt_to_text s.current_stmt;
        ];
        [
          "next_frame_id";
          string_of_int (Frame_id.Supply.to_int s.next_frame_id);
        ];
      ];
    text_table ~title:"env"
      ~columns:[ "var"; "location"; "value" ]
      (List.map
         (fun (n, l) ->
           [
             Utils_names.Identifier.to_string n;
             Runtime_value.location_text l;
             Runtime_value.print
               (try store_lookup s l with Failure _ -> Null_ptr);
           ])
         (Utils_base.Assoc.sorted s.env));
    container ~title:"store"
      (List.map
         (fun f ->
           assoc_table
             ~title:("frame " ^ string_of_int (Frame_id.to_int f.frame_id))
             ~columns:[ "location"; "value" ] Runtime_value.print
             (List.map
                (fun (l, v) -> (Runtime_value.location_text l, v))
                (Utils_base.Assoc.sorted f.values)))
         (List.rev s.store));
    container ~title:"inner_k"
      (List.map (fun st -> text (Code.stmt_to_text st)) s.inner_k);
    text_table ~title:"fn_k"
      ~columns:[ "depth"; "function"; "resume_len" ]
      (List.mapi
         (fun i f ->
           [
             string_of_int i;
             Utils_names.Identifier.to_string f.function_name;
             string_of_int (List.length f.inner_k);
           ])
         s.fn_k);
  ]
