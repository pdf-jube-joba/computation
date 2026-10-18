open Types
module L = Utils_parsing.Parser

let get = Utils_base.Error.get

type t = ainput

let print_value (value : flow_value) =
  match value with
  | Num n -> N.to_string n
  | Code_label l -> ":" ^ Utils_names.Identifier.to_string l
  | Key (Static l) -> "k:@" ^ Utils_names.Identifier.to_string l
  | Key (Stack i) -> Printf.sprintf "k:s[%d]" i
  | Key (Heap (h, i)) -> Printf.sprintf "k:h[%d][%d]" (Heap_id.to_int h) i

let parse_value text =
  L.protect (fun () ->
      let s = String.trim text in
      let prefix p = String.starts_with ~prefix:p s in
      if prefix ":" then
        let label = Utils_base.String_ext.strip_prefix ":" s in
        Code_label (get (Utils_names.Identifier.make label))
      else if prefix "k:@" then
        let label = Utils_base.String_ext.strip_prefix "k:@" s in
        Key (Static (get (Utils_names.Identifier.make label)))
      else if prefix "k:s[" then (
        let p = L.create s in
        L.expect p "k:s";
        let index = L.group p L.Brackets L.integer in
        L.expect_eof p;
        Key (Stack index))
      else if prefix "k:h[" then (
        let p = L.create s in
        L.expect p "k:h";
        let handle = L.group p L.Brackets L.integer in
        let index = L.group p L.Brackets L.integer in
        L.expect_eof p;
        Key (Heap (get (Heap_id.of_int handle), index)))
      else Num (get (N.of_string s)))

let parse text =
  Utils_codec.Environment_text.parse
    ~invalid_line:(fun line -> "Invalid static input line: " ^ line)
    parse_value text
  |> Result.map Utils_base.Assoc.sorted

let print env =
  Utils_codec.Environment_text.print ~trailing_newline:true print_value
    (Utils_base.Assoc.sorted env)
