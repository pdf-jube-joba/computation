let fold_lines ?(invalid_line = fun line -> "Invalid env line: " ^ line) f init
    text =
  Lexer.protect (fun () ->
      List.fold_left
        (fun env line ->
          let line = String.trim line in
          if line = "" then env
          else
            match String.index_opt line '=' with
            | None -> failwith (invalid_line line)
            | Some i ->
                let name = String.trim (String.sub line 0 i) in
                let value =
                  String.trim
                    (String.sub line (i + 1) (String.length line - i - 1))
                in
                f env name value)
        init
        (String.split_on_char '\n' text))

let parse ?invalid_line parse_value text =
  let get = function Ok value -> value | Error error -> failwith error in
  fold_lines ?invalid_line
    (fun env name value ->
      let name = get (Identifier.make name) in
      let value = get (parse_value value) in
      Assoc.set name value env)
    [] text

let print ?(trailing_newline = false) value_text env =
  let lines =
    List.map (fun (name, value) -> name ^ " = " ^ value_text value) env
  in
  String.concat "\n" lines
  ^ if trailing_newline && lines <> [] then "\n" else ""
