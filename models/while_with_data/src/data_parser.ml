type token =
  | Word of string
  | Character of string
  | Sym of char
  | Op of string
  | Eof

type t = { tokens : token array; mutable pos : int }

let fail = failwith

let create s =
  let n = String.length s in
  let alpha = Utils.Identifier.is_start in
  let alnum = Utils.Identifier.is_continue in
  let rec scan i acc =
    if i = n then Array.of_list (List.rev (Eof :: acc))
    else
      match s.[i] with
      | c when Utils.String_ext.is_whitespace c -> scan (i + 1) acc
      | '\'' ->
          let c, j = Utils.Char_literal.read s i in
          scan j (Character c :: acc)
      | c when alpha c ->
          let j = Utils.String_ext.skip_while alnum s (i + 1) in
          scan j (Word (String.sub s i (j - i)) :: acc)
      | (':' | '+' | '=') as c ->
          let other = if c = ':' then '=' else c in
          if i + 1 >= n || s.[i + 1] <> other then fail "Invalid operator";
          scan (i + 2) (Op (String.sub s i 2) :: acc)
      | ('(' | ')' | '[' | ']' | '{' | '}' | ',' | ';') as c ->
          scan (i + 1) (Sym c :: acc)
      | c -> fail ("Unexpected character: " ^ String.make 1 c)
  in
  { tokens = scan 0 []; pos = 0 }

let peek p = p.tokens.(p.pos)

let next p =
  let t = peek p in
  if t <> Eof then p.pos <- p.pos + 1;
  t

let eat p t =
  if peek p = t then (
    ignore (next p);
    true)
  else false

let expect p t = if not (eat p t) then fail "Unexpected token"

let ident p =
  match next p with
  | Word n -> (
      match Utils.Identifier.make n with Ok n -> n | Error e -> fail e)
  | _ -> fail "Expected identifier"

let character p =
  match next p with Character c -> c | _ -> fail "Expected character literal"

let parse f text =
  Utils.Lexer.protect (fun () ->
      let p = create text in
      let x = f p in
      expect p Eof;
      x)

let set env name value = Utils.Assoc.set name value env
let env parse_value text = Utils.Environment.parse parse_value text

let env_text value_text env =
  Utils.Environment.print ~trailing_newline:true value_text env
