open Types

let render ~highlight s =
  let name = Code.state_name s.definition in
  let signs, head = Utils_data.Tape.into_vec s.tape in
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
      Utils_runtime.Render.highlight ~title:"tape" ~index:head Sign.print signs;
    ]
