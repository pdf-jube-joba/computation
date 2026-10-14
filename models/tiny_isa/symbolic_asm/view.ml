open Types

let render s =
  let next =
    if s.pc >= 0 && s.pc < Array.length s.insts then
      Tiny_isa_shared.Instruction.inst_text Code.raw_value_to_text
        s.insts.(s.pc)
    else "halt"
  in
  let data_rows =
    Array.to_list
      (Array.mapi
         (fun addr v ->
           let labels =
             Option.value ~default:[]
               (List.assoc_opt addr s.debug_info.data_labels_by_addr)
           in
           [
             string_of_int addr;
             (if labels = [] then "-" else String.concat ", " labels);
             N.to_string v;
           ])
         s.data_memory)
  in
  Utils_runtime.Render.state
    [
      Utils_runtime.Render.text_table ~title:"status"
        ~columns:[ "field"; "value" ]
        [
          [ "pc"; string_of_int s.pc ];
          [ "flag"; string_of_bool s.flag ];
          [ "next"; next ];
          [ "halted"; string_of_bool s.halted ];
        ];
      Tiny_isa_shared.View.registers s.regs;
      Utils_runtime.Render.text_table ~title:"data"
        ~columns:[ "addr"; "label"; "value" ]
        data_rows;
    ]
