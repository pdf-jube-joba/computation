type 'a t = { register : 'a array; memory : 'a list } [@@deriving yojson]

module Make (Value : Text.S) = struct
  let value = Utils_base.Error.get

  type nonrec t = Value.t t

  let parse ~register_count text =
    Utils_parsing.Parser.protect (fun () ->
        let expected = string_of_int register_count ^ " registers" in
        match Utils_base.String_ext.lines text with
        | [] -> failwith ("Expected first line with " ^ expected)
        | reg_line :: mem ->
            let parts = Utils_base.String_ext.split_nonempty ',' reg_line in
            if List.length parts <> register_count then
              failwith
                ("Expected " ^ expected ^ " on the first line (comma separated)");
            {
              register =
                Array.of_list (List.map (fun s -> value (Value.parse s)) parts);
              memory = List.map (fun s -> value (Value.parse s)) mem;
            })

  let print (image : t) =
    String.concat ", " (List.map Value.print (Array.to_list image.register))
    ^
    if image.memory = [] then ""
    else "\n" ^ String.concat "\n" (List.map Value.print image.memory)
end

module Natural = Make (Text.Natural)
