let env value_text bindings =
  Utils_runtime.Render.keyed_table ~key_text:Utils_names.Identifier.to_string
    ~title:"env" ~columns:[ "name"; "value" ] value_text (List.rev bindings)

let continuation ~code ~mode ~focus ~bindings ~frame_text frames =
  let open Utils_runtime.Render in
  state
    [
      text ~title:"code" code;
      text_table ~title:"state" ~columns:[ "field"; "value" ]
        [ [ "mode"; mode ]; [ "focus"; focus ] ];
      env Closure_value.value_text bindings;
      indexed_table ~title:"kont" ~columns:[ "depth"; "frame" ] frame_text
        frames;
    ]
