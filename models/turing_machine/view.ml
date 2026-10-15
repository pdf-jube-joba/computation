open Types

let render_tape = Utils_runtime.Data_view.tape Sign.print

let render ~highlight s =
  let name = Code.state_name s.definition in
  Utils_runtime.Render.state
    [
      Utils_runtime.Render.text_table ~title:"code" ?highlight
        ~columns:
          [ "key_sign"; "key_state"; "next_sign"; "next_state"; "direction" ]
        (List.map
           (fun ((a, b), (c, d, e)) ->
             [
               Sign.print a;
               Utils_names.Identifier.to_string (name b);
               Sign.print c;
               Utils_names.Identifier.to_string (name d);
               Direction.print e;
             ])
           s.definition.code);
      Utils_runtime.Render.text ~title:"state"
        (Utils_names.Identifier.to_string (name s.current_state));
      render_tape s.tape;
    ]
