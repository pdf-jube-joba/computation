(* Grammar adapter for the shared lexer; this dialect permits hyphenated names. *)
type token =
  | Keyword of string
  | Word of string
  | Character of string
  | Sym of char
  | Op of string
  | Eof

type t = { tokens : token array; mutable pos : int }

let fail = failwith

let char_text = function
  | "\n" -> "'\\n'"
  | "\r" -> "'\\r'"
  | "\t" -> "'\\t'"
  | "\\" -> "'\\\\'"
  | "'" -> "'\\''"
  | c -> "'" ^ c ^ "'"

let create text =
  let raw = Utils.Lexer.tokenize ~identifier_extra:(( = ) '-') text in
  let rec tokens i acc =
    if i = Array.length raw then Array.of_list (List.rev acc)
    else
      let token =
        match fst raw.(i) with
        | Utils.Lexer.Ident s -> Word s
        | Utils.Lexer.Keyword s -> Keyword s
        | Utils.Lexer.Character c -> Character c
        | Utils.Lexer.Eof -> Eof
        | Utils.Lexer.Symbol ((':' | '+' | '=') as c)
          when i + 1 < Array.length raw
               && fst raw.(i + 1)
                  = Utils.Lexer.Symbol (if c = ':' then '=' else c) ->
            Op (String.make 1 c ^ String.make 1 (if c = ':' then '=' else c))
        | Utils.Lexer.Symbol c -> Sym c
        | _ -> fail "Unexpected token"
      in
      tokens (i + match token with Op _ -> 2 | _ -> 1) (token :: acc)
  in
  { tokens = tokens 0 []; pos = 0 }

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
      let text = Utils.Lexer.without_comments text in
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
