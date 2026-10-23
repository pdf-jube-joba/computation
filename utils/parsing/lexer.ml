type token =
  | Ident of string
  | Keyword of string
  | Number of string
  | String_lit of string
  | Character_lit of string
  | Symbol of char
  | Eof

exception Error of string

(* Some source languages close block comments at the first terminator. *)
let skip_block_comment ?(nested_comments = true) text i =
  let j = ref (i + 2) and depth = ref 1 in
  while !j < String.length text && !depth > 0 do
    if nested_comments && Utils_base.String_ext.starts_at text !j "/*" then (
      incr depth;
      j := !j + 2)
    else if Utils_base.String_ext.starts_at text !j "*/" then (
      decr depth;
      j := !j + 2)
    else incr j
  done;
  if !depth <> 0 then
    raise (Error (Printf.sprintf "Unclosed comment at byte %d" i));
  !j

let tokenize ?(nested_comments = true) ?(character_literals = false) text =
  let n = String.length text in
  let alpha = Utils_names.Identifier.is_start in
  let digit = function '0' .. '9' -> true | _ -> false in
  let has = Utils_base.String_ext.starts_at text in
  let rec scan i acc =
    if i >= n then Array.of_list (List.rev ((Eof, n) :: acc))
    else
      match text.[i] with
      | c when Utils_base.String_ext.is_whitespace c -> scan (i + 1) acc
      | '/' when has i "//" ->
          scan (Utils_base.String_ext.skip_line text (i + 2)) acc
      | '/' when has i "/*" ->
          scan (skip_block_comment ~nested_comments text i) acc
      | '\'' when character_literals ->
          let c, j = Char_literal.read text i in
          scan j ((Character_lit c, i) :: acc)
      | '"' ->
          let j = ref (i + 1) and escaped = ref false in
          while !j < n && (!escaped || text.[!j] <> '"') do
            if !escaped then escaped := false
            else if text.[!j] = '\\' then escaped := true;
            incr j
          done;
          if !j = n then
            raise (Error (Printf.sprintf "Unclosed string at byte %d" i));
          let literal = String.sub text i (!j - i + 1) in
          let value =
            try
              match Yojson.Safe.from_string literal with
              | `String s -> s
              | _ -> assert false
            with Yojson.Json_error e -> raise (Error e)
          in
          scan (!j + 1) ((String_lit value, i) :: acc)
      | '\\' when i + 1 < n && alpha text.[i + 1] ->
          let j =
            Utils_base.String_ext.skip_while Utils_names.Identifier.is_continue
              text (i + 2)
          in
          let word = String.sub text (i + 1) (j - i - 1) in
          scan j ((Keyword word, i) :: acc)
      | c when alpha c || digit c ->
          let is_number = digit c in
          let predicate =
            if is_number then digit else Utils_names.Identifier.is_continue
          in
          let j = Utils_base.String_ext.skip_while predicate text (i + 1) in
          let word = String.sub text i (j - i) in
          scan j (((if is_number then Number word else Ident word), i) :: acc)
      | c when Char.code c < 128 && c <> '\000' ->
          scan (i + 1) ((Symbol c, i) :: acc)
      | _ -> raise (Error (Printf.sprintf "Unexpected character at byte %d" i))
  in
  scan 0 []
