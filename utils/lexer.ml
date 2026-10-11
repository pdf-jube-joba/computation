type token =
  | Ident of string
  | Number of string
  | String_lit of string
  | Character_lit of string
  | Symbol of char
  | Eof

type t = { tokens : (token * int) array; mutable pos : int }

exception Error of string

let error p message =
  let _, offset = p.tokens.(min p.pos (Array.length p.tokens - 1)) in
  raise (Error (Printf.sprintf "%s at byte %d" message offset))

(* Some source languages close block comments at the first terminator. *)
let skip_block_comment ?(nested_comments = true) text i =
  let j = ref (i + 2) and depth = ref 1 in
  while !j < String.length text && !depth > 0 do
    if nested_comments && String_ext.starts_at text !j "/*" then (
      incr depth;
      j := !j + 2)
    else if String_ext.starts_at text !j "*/" then (
      decr depth;
      j := !j + 2)
    else incr j
  done;
  if !depth <> 0 then
    raise (Error (Printf.sprintf "Unclosed comment at byte %d" i));
  !j

let tokenize ?(nested_comments = true) ?(character_literals = false) text =
  let n = String.length text in
  let alpha = Identifier.is_start in
  let digit = function '0' .. '9' -> true | _ -> false in
  let has = String_ext.starts_at text in
  let rec scan i acc =
    if i >= n then Array.of_list (List.rev ((Eof, n) :: acc))
    else
      match text.[i] with
      | c when String_ext.is_whitespace c -> scan (i + 1) acc
      | '/' when has i "//" -> scan (String_ext.skip_line text (i + 2)) acc
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
      | c when alpha c || digit c ->
          let is_number = digit c in
          let predicate = if is_number then digit else Identifier.is_continue in
          let j = String_ext.skip_while predicate text (i + 1) in
          let word = String.sub text i (j - i) in
          scan j (((if is_number then Number word else Ident word), i) :: acc)
      | c when Char.code c < 128 && c <> '\000' ->
          scan (i + 1) ((Symbol c, i) :: acc)
      | _ -> raise (Error (Printf.sprintf "Unexpected character at byte %d" i))
  in
  scan 0 []

let create ?(nested_comments = true) ?(character_literals = false) text =
  { tokens = tokenize ~nested_comments ~character_literals text; pos = 0 }

let peek p = fst p.tokens.(p.pos)

let next p =
  let token = peek p in
  if token <> Eof then p.pos <- p.pos + 1;
  token

let eof p = peek p = Eof
let expect_eof p = if not (eof p) then error p "Unexpected trailing tokens"

let eat_symbol p c =
  if peek p = Symbol c then (
    ignore (next p);
    true)
  else false

let expect_symbol p c =
  if not (eat_symbol p c) then error p (Printf.sprintf "Expected '%c'" c)

let eat_keyword p word =
  if peek p = Ident word then (
    ignore (next p);
    true)
  else false

let expect_keyword p word =
  if not (eat_keyword p word) then error p ("Expected " ^ word)

let identifier p =
  match next p with Ident s -> s | _ -> error p "Expected identifier"

let natural p =
  match next p with
  | Number s -> Number.normalize s
  | _ -> error p "Expected natural number"

let integer p =
  match int_of_string_opt (natural p) with
  | Some n when n >= 0 -> n
  | _ -> error p "Integer out of range"

let string_literal p =
  match next p with String_lit s -> s | _ -> error p "Expected string literal"

let character_literal p =
  match next p with
  | Character_lit c -> c
  | _ -> error p "Expected character literal"

let eat p text =
  let saved = p.pos in
  let wanted = tokenize text in
  let rec loop i =
    if i = Array.length wanted - 1 then true
    else if peek p = fst wanted.(i) then (
      ignore (next p);
      loop (i + 1))
    else false
  in
  if loop 0 then true
  else (
    p.pos <- saved;
    false)

let expect p text = if not (eat p text) then error p ("Expected " ^ text)

let protect f =
  try Ok (f ())
  with Error message | Failure message | Invalid_argument message ->
    Result.Error message

let parse_with create f text =
  protect (fun () ->
      let p = create text in
      let value = f p in
      expect_eof p;
      value)

let parse ?(nested_comments = true) ?(character_literals = false) f text =
  parse_with (create ~nested_comments ~character_literals) f text

(* Consume the closing symbol. When supplied, skip_separator allows separators
   to be omitted, repeated, or placed before/after the items. *)
let many_until ?skip_separator p ~closing item =
  let rec skip () =
    match skip_separator with Some c when eat_symbol p c -> skip () | _ -> ()
  in
  let rec loop acc =
    skip ();
    if eat_symbol p closing then List.rev acc
    else
      let value = item p in
      loop (value :: acc)
  in
  loop []

let rec separated p ~separator item =
  let first = item p in
  if eat_symbol p separator then first :: separated p ~separator item
  else [ first ]

let separated_until ?(allow_trailing = false) p ~separator ~closing item =
  let rec loop acc =
    let value = item p in
    let acc = value :: acc in
    if eat_symbol p separator then
      if allow_trailing && eat_symbol p closing then List.rev acc else loop acc
    else (
      expect_symbol p closing;
      List.rev acc)
  in
  if eat_symbol p closing then [] else loop []
