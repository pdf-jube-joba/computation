module Json = Utils_codec.Json

let get = Utils_base.Error.get
let output json = print_endline (Json.encode json)

let stdin_text () =
  let buffer = Buffer.create 128 in
  (try
     while true do
       Buffer.add_string buffer (input_line stdin);
       Buffer.add_char buffer '\n'
     done
   with End_of_file -> ());
  Buffer.contents buffer

let read_block () =
  let buffer = Buffer.create 128 in
  let rec lines () =
    match input_line stdin with
    | ":end" -> Buffer.contents buffer
    | line ->
        Buffer.add_string buffer line;
        Buffer.add_char buffer '\n';
        lines ()
    | exception End_of_file -> Buffer.contents buffer
  in
  lines ()

let expand_input = function ":begin" -> read_block () | input -> input

let prompt label =
  print_string (label ^ "> ");
  flush stdout;
  match input_line stdin with
  | line -> expand_input line
  | exception End_of_file -> ""
