open Types

let render s =
  let current_block =
    if s.current_block >= 0 && s.current_block < Array.length s.compiled.blocks
    then Some s.compiled.blocks.(s.current_block)
    else None
  in
  [
    Utils_runtime.Render.text_table ~title:"status"
      ~columns:[ "field"; "value" ]
      [
        [ "block"; string_of_int s.current_block ];
        [
          "label";
          (match current_block with
          | Some b -> Utils_names.Identifier.to_string b.label
          | None -> "<invalid>");
        ];
        [ "halted"; string_of_bool s.halted ];
      ];
    Utils_runtime.Render.indexed_table ~title:"block"
      ~columns:[ "index"; "stmt" ] Code.stmt_to_text
      (match current_block with None -> [] | Some b -> b.stmts);
    Utils_runtime.Render.indexed_table ~title:"vregs"
      ~columns:[ "vreg"; "value" ]
      ~index_text:(fun i -> "v" ^ string_of_int i)
      N.to_string (Array.to_list s.vregs);
    Utils_runtime.Render.indexed_table ~title:"memory"
      ~columns:[ "addr"; "value" ] N.to_string (Array.to_list s.memory);
  ]
