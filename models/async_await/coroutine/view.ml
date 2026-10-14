open Runtime_types

let render s =
  let open Utils_runtime.Render in
  state
    [
      assoc_table ~title:"status" ~columns:[ "field"; "value" ] Fun.id
        [
          ("agents", string_of_int (Array.length s.workers));
          ("queue_len", string_of_int (List.length s.queue));
          ("task_count", string_of_int (Array.length s.tasks));
        ];
      container ~title:"env" ~orientation:Utils_runtime.Render.Horizontal
        [
          assoc_table ~title:"vars" N.to_string s.env.vars;
          assoc_table ~title:"task_ids" ~columns:[ "id"; "task" ]
            (function None -> "none" | Some n -> task_text n)
            (List.map (fun (n, id) -> ("$" ^ n, id)) s.env.task_ids);
        ];
      indexed_table ~title:"workers" ~columns:[ "agent"; "task" ]
        (function None -> "idle" | Some n -> task_text n)
        (Array.to_list s.workers);
      indexed_table ~title:"queue" ~columns:[ "slot"; "task" ] task_text s.queue;
      container ~title:"tasks"
        (Array.to_list
           (Array.mapi
              (fun id stack ->
                text_table
                  ~title:(Printf.sprintf "task %d" id)
                  ~columns:[ "depth"; "function"; "pc" ]
                  (List.mapi
                     (fun i f ->
                       [ string_of_int i; f.function_name; string_of_int f.pc ])
                     (List.rev stack)))
              s.tasks));
    ]
