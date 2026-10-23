open Types
module L = Utils_parsing.Parser

let get = Utils_base.Error.get

type t = ainput

let print_value (value : flow_value) =
  match value with
  | Num n -> N.to_string n
  | Code_label l -> ":" ^ Utils_names.Identifier.to_string l
  | Key (Static l) -> "\\static @" ^ Utils_names.Identifier.to_string l
  | Key (Stack i) -> Printf.sprintf "\\stack[%d]" i
  | Key (Heap (h, i)) -> Printf.sprintf "\\heap[%d][%d]" (Heap_id.to_int h) i

let parse_value =
  L.parse (fun p ->
      if L.bump_if_symbol p ':' then Code_label (L.identifier p)
      else if L.bump_if_keyword p "static" then (
        L.expect_symbol p '@';
        Key (Static (L.identifier p)))
      else if L.bump_if_keyword p "stack" then
        Key (Stack (L.group p L.Brackets L.integer))
      else if L.bump_if_keyword p "heap" then
        let handle = L.group p L.Brackets L.integer in
        let index = L.group p L.Brackets L.integer in
        Key (Heap (get (Heap_id.of_int handle), index))
      else Num (L.natural p))

let parse text =
  Utils_data.Environment.parse
    ~invalid_line:(fun line -> "Invalid static input line: " ^ line)
    parse_value text
  |> Result.map Utils_base.Assoc.sorted

let print env =
  Utils_data.Environment.print ~trailing_newline:true print_value
    (Utils_base.Assoc.sorted env)
