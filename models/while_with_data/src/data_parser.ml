(* Both languages have character literals, including Unicode scalars.
   Keep that lexer distinct from Utils.Lexer. *)
type token =
  | Word of string
  | Character of string
  | Sym of char
  | Op of string
  | Eof

type t = { tokens : token array; mutable pos : int }

let fail = failwith

let scalar_length s i =
  let c = Char.code s.[i] in
  if c < 128 then 1
  else if c land 0xe0 = 0xc0 then 2
  else if c land 0xf0 = 0xe0 then 3
  else if c land 0xf8 = 0xf0 then 4
  else fail "Invalid UTF-8 character"

let scalar s i =
  let n = scalar_length s i in
  if i + n > String.length s then fail "Truncated UTF-8 character";
  for j = i + 1 to i + n - 1 do
    if Char.code s.[j] land 0xc0 <> 0x80 then fail "Invalid UTF-8 character"
  done;
  (String.sub s i n, i + n)

let char_text = function
  | "\n" -> "'\\n'"
  | "\r" -> "'\\r'"
  | "\t" -> "'\\t'"
  | "\\" -> "'\\\\'"
  | "'" -> "'\\''"
  | c -> "'" ^ c ^ "'"

let create s =
  let n = String.length s in
  let alpha = Utils.Identifier.is_start in
  let alnum = Utils.Identifier.is_continue in
  let rec scan i acc =
    if i = n then Array.of_list (List.rev (Eof :: acc))
    else
      match s.[i] with
      | ' ' | '\n' | '\r' | '\t' -> scan (i + 1) acc
      | '\'' ->
          let c, j =
            if i + 1 >= n then fail "Unterminated character literal"
            else if s.[i + 1] = '\\' then (
              if i + 2 >= n then fail "Unterminated escape";
              let c, k = scalar s (i + 2) in
              ( (match c with "n" -> "\n" | "r" -> "\r" | "t" -> "\t" | c -> c),
                k ))
            else scalar s (i + 1)
          in
          if j >= n || s.[j] <> '\'' then fail "Expected closing quote";
          scan (j + 1) (Character c :: acc)
      | c when alpha c ->
          let j = ref (i + 1) in
          while !j < n && alnum s.[!j] do
            incr j
          done;
          scan !j (Word (String.sub s i (!j - i)) :: acc)
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

let set env name value =
  if List.mem_assoc name env then
    List.map (fun (k, v) -> (k, if k = name then value else v)) env
  else env @ [ (name, value) ]

let env parse_value text =
  Utils.Lexer.protect (fun () ->
      List.fold_left
        (fun env line ->
          let line = String.trim line in
          if line = "" then env
          else
            match String.index_opt line '=' with
            | None -> fail ("Invalid env line: " ^ line)
            | Some i ->
                let name = String.trim (String.sub line 0 i) in
                let name =
                  match Utils.Identifier.make name with
                  | Ok n -> n
                  | Error e -> fail e
                in
                let value =
                  String.trim
                    (String.sub line (i + 1) (String.length line - i - 1))
                in
                let value =
                  match parse_value value with Ok v -> v | Error e -> fail e
                in
                set env name value)
        []
        (String.split_on_char '\n' text))

let env_text value_text env =
  String.concat ""
    (List.map (fun (n, v) -> n ^ " = " ^ value_text v ^ "\n") env)

let render stmt value_text env =
  let open Utils.Render in
  state
    [
      text ~title:"stmt" stmt;
      table ~title:"env"
        ~columns:[ text "var"; text "value" ]
        (List.map (fun (n, v) -> [ text n; text (value_text v) ]) env);
    ]
