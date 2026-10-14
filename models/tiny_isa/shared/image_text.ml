module N = Utils_base.Number

let value = Utils_base.Error.get

type t = N.t array * N.t list

let parse text =
  Utils_parsing.Parser.protect (fun () ->
      match Utils_base.String_ext.lines text with
      | [] -> failwith "Expected first line with 8 registers"
      | reg_line :: mem ->
          let parts = Utils_base.String_ext.split_nonempty ',' reg_line in
          if List.length parts <> 8 then
            failwith "Expected 8 registers on the first line (comma separated)";
          ( Array.of_list
              (List.map (fun s -> value (N.of_string_with_plus s)) parts),
            List.map (fun s -> value (N.of_string_with_plus s)) mem ))

let print (regs, mem) =
  String.concat ", " (List.map N.to_string (Array.to_list regs))
  ^ if mem = [] then "" else "\n" ^ Numbers_text.print mem
