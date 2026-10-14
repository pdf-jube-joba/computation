open Runtime_types

let format_final_state s f =
  let values indent f =
    String.concat ""
      (List.map
         (fun (l, v) ->
           indent
           ^ Runtime_value.location_text l
           ^ " = " ^ Runtime_value.print v ^ "\n")
         (Utils_base.Assoc.sorted f.values))
  in
  "halted\ncurrent_function: "
  ^ Utils_names.Identifier.to_string s.current_function
  ^ "\nfinished_frame: "
  ^ string_of_int (Frame_id.to_int f.frame_id)
  ^ "\n" ^ values "  " f ^ "remaining_store:\n"
  ^
  if s.store = [] then "  <empty>\n"
  else
    String.concat ""
      (List.map
         (fun f ->
           "  frame "
           ^ string_of_int (Frame_id.to_int f.frame_id)
           ^ ":\n" ^ values "    " f)
         (List.rev s.store))
