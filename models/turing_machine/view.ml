open Types
open Execution

let render_tape t =
  let signs, pos = Tape.into_vec t in
  Utils_runtime.Render.container ~title:"tape"
    ~orientation:Utils_runtime.Render.Horizontal
    (List.mapi
       (fun i s ->
         if i = pos then
           Utils_runtime.Render.text ~class_name:"highlight" (Sign.print s)
         else Utils_runtime.Render.text (Sign.print s))
       signs)

let render s =
  let index = Option.map fst (next_code s) in
  let name = Code.state_name s.definition in
  Utils_runtime.Render.state
    [
      Utils_runtime.Render.text_table ~title:"code" ?highlight:index
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
