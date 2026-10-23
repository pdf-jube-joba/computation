open Types
open Resolution

let runtime_text s v = Input_text.print_value (named_value s.resolution v)

let render s =
  let module R = Utils_runtime.Render in
  let r = List.nth_opt s.code.regions s.current_region in
  let b = Option.bind r (fun r -> List.nth_opt r.blocks s.current_block) in
  [
    R.text ~title:"current_region"
      (":"
      ^ Option.fold ~none:"<invalid>"
          ~some:(fun (r : region) -> Utils_names.Identifier.to_string r.label)
          r);
    R.text ~title:"current_block"
      (":"
      ^ Option.fold ~none:"<invalid>"
          ~some:(fun (b : block) -> Utils_names.Identifier.to_string b.label)
          b);
    R.text ~title:"current_line" (string_of_int s.current_line);
    R.text ~title:"halted" (string_of_bool s.halted);
    R.assoc_table ~title:"vregs" ~columns:[ "vreg"; "value" ] (runtime_text s)
      (List.map
         (fun (k, v) ->
           ( "%"
             ^ Utils_names.Identifier.to_string
                 (name_of_id s.resolution.registers k),
             v ))
         (Utils_base.Assoc.sorted s.vregs));
    R.text_table ~title:"static" ~columns:[ "label"; "value" ]
      (List.map
         (fun (d : static_def) ->
           [
             "@" ^ Utils_names.Identifier.to_string d.label;
             runtime_text s
               (Option.value
                  (List.assoc_opt
                     (id_of_name s.resolution.data d.label)
                     s.static_mem)
                  ~default:(Num N.zero));
           ])
         s.code.statics);
    R.indexed_table ~title:"stack" (runtime_text s) s.stack;
    R.text_table ~title:"heap"
      ~columns:[ "handle"; "index"; "value" ]
      (List.concat_map
         (fun (h, vs) ->
           List.mapi
             (fun i v ->
               [
                 string_of_int (Heap_id.to_int h);
                 string_of_int i;
                 runtime_text s v;
               ])
             vs)
         (Utils_base.Assoc.sorted s.heap));
  ]
