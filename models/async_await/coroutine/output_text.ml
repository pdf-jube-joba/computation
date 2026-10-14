open Runtime_types

let value = Utils_base.Error.get

type t = foutput

let parse_task_id text =
  if String.lowercase_ascii text = "none" then None
  else
    Some
      (value (Task_id.of_int (value (Utils_codec.Text.Natural_int.parse text))))

let print_task_id = function None -> "none" | Some n -> task_text n

let parse text =
  Utils_codec.Environment_text.fold_lines
    ~invalid_line:(fun line -> "invalid env line: " ^ line)
    (fun env name v ->
      if String.starts_with ~prefix:"$" name then
        let name = Utils_base.String_ext.strip_prefix "$" name in
        let name = value (Utils_names.Identifier.make name) in
        let id = parse_task_id v in
        { env with task_ids = Utils_base.Assoc.set_sorted name id env.task_ids }
      else
        let name = value (Utils_names.Identifier.make name) in
        {
          env with
          vars =
            Utils_base.Assoc.set_sorted name
              (value (N.of_string_with_plus v))
              env.vars;
        })
    { vars = []; task_ids = [] }
    text

let print env =
  Utils_codec.Environment_text.print ~trailing_newline:true N.to_string env.vars
  ^ Utils_codec.Environment_text.print_lines ~trailing_newline:true
      print_task_id
      (List.map
         (fun (n, v) -> ("$" ^ Utils_names.Identifier.to_string n, v))
         env.task_ids)
