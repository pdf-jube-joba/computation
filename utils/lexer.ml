type token =
  | Ident of string
  | Keyword of string
  | Character of string
  | Number of string
  | String_lit of string
  | Symbol of char
  | Eof

type t = { tokens : (token * int) array; mutable pos : int }

exception Error of string

let error p message =
  let _, offset = p.tokens.(min p.pos (Array.length p.tokens - 1)) in
  raise (Error (Printf.sprintf "%s at byte %d" message offset))

(* Comments are replaced by spaces so byte offsets and line boundaries survive.
   Quoted literals are copied unchanged, including comment-looking contents. *)
let without_comments text =
  let n = String.length text in
  let bytes = Bytes.of_string text in
  let has i prefix =
    i + String.length prefix <= n
    && String.sub text i (String.length prefix) = prefix
  in
  let mask i =
    if text.[i] <> '\n' && text.[i] <> '\r' then Bytes.set bytes i ' '
  in
  let rec scan i =
    if i < n then
      if has i "//" then (
        let j = ref i in
        while !j < n && text.[!j] <> '\n' do
          mask !j;
          incr j
        done;
        scan !j)
      else if has i "/*" then (
        let j = ref (i + 2) and depth = ref 1 in
        mask i;
        mask (i + 1);
        while !j < n && !depth > 0 do
          if has !j "/*" then (
            incr depth;
            mask !j;
            mask (!j + 1);
            j := !j + 2)
          else if has !j "*/" then (
            decr depth;
            mask !j;
            mask (!j + 1);
            j := !j + 2)
          else (
            mask !j;
            incr j)
        done;
        if !depth <> 0 then
          raise (Error (Printf.sprintf "Unclosed comment at byte %d" i));
        scan !j)
      else if text.[i] = '"' then quoted '"' (i + 1)
      else if text.[i] = '\'' then (
        (* Only a complete character literal opens a quote. Apostrophes in
           identifiers remain ordinary characters for dialect lexers. *)
        let j = ref (i + 1) in
        if !j < n && text.[!j] = '\\' then j := !j + 2
        else if !j < n then (
          incr j;
          while !j < n && Char.code text.[!j] land 0xc0 = 0x80 do
            incr j
          done);
        if !j < n && text.[!j] = '\'' then scan (!j + 1) else scan (i + 1))
      else scan (i + 1)
  and quoted quote i =
    if i < n then
      if text.[i] = '\\' then quoted quote (i + 2)
      else if text.[i] = quote then scan (i + 1)
      else quoted quote (i + 1)
  in
  scan 0;
  Bytes.to_string bytes

let scalar_length s i =
  let c = Char.code s.[i] in
  let length, minimum =
    if c < 128 then (1, 0)
    else if c >= 0xc2 && c <= 0xdf then (2, 0x80)
    else if c >= 0xe0 && c <= 0xef then (3, 0x800)
    else if c >= 0xf0 && c <= 0xf4 then (4, 0x10000)
    else raise (Error "Invalid UTF-8 character")
  in
  if i + length > String.length s then raise (Error "Truncated UTF-8 character");
  let value =
    ref
      (c land match length with 1 -> 0x7f | 2 -> 0x1f | 3 -> 0x0f | _ -> 0x07)
  in
  for j = i + 1 to i + length - 1 do
    let byte = Char.code s.[j] in
    if byte land 0xc0 <> 0x80 then raise (Error "Invalid UTF-8 character");
    value := (!value lsl 6) lor (byte land 0x3f)
  done;
  if
    !value < minimum || !value > 0x10ffff
    || (!value >= 0xd800 && !value <= 0xdfff)
  then raise (Error "Invalid Unicode scalar");
  length

let tokenize ?(keywords = true) ?(identifier_extra = fun _ -> false) text =
  let text = without_comments text in
  let n = String.length text in
  let alpha = function 'a' .. 'z' | 'A' .. 'Z' | '_' -> true | _ -> false in
  let digit = function '0' .. '9' -> true | _ -> false in
  let word_end start extra =
    let j = ref (start + 1) in
    while !j < n && (alpha text.[!j] || digit text.[!j] || extra text.[!j]) do
      incr j
    done;
    !j
  in
  let rec scan i acc =
    if i >= n then Array.of_list (List.rev ((Eof, n) :: acc))
    else
      match text.[i] with
      | ' ' | '\n' | '\r' | '\t' -> scan (i + 1) acc
      | '\\' when keywords ->
          if i + 1 >= n || not (alpha text.[i + 1]) then
            raise
              (Error
                 (Printf.sprintf "Expected keyword after backslash at byte %d" i));
          let j = word_end (i + 1) (( = ) '-') in
          scan j ((Keyword (String.sub text (i + 1) (j - i - 1)), i) :: acc)
      | '\'' ->
          let value, j =
            if i + 1 >= n then raise (Error "Unterminated character literal")
            else if text.[i + 1] = '\\' then (
              if i + 2 >= n then raise (Error "Unterminated character escape");
              let c =
                match text.[i + 2] with
                | 'n' -> "\n"
                | 'r' -> "\r"
                | 't' -> "\t"
                | '\\' -> "\\"
                | '\'' -> "'"
                | _ -> raise (Error "Invalid character escape")
              in
              (c, i + 3))
            else
              let len = scalar_length text (i + 1) in
              (String.sub text (i + 1) len, i + 1 + len)
          in
          if j >= n || text.[j] <> '\'' then
            raise (Error "Expected closing character quote");
          scan (j + 1) ((Character value, i) :: acc)
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
      | c when digit c ->
          let j = ref (i + 1) in
          while !j < n && digit text.[!j] do
            incr j
          done;
          scan !j ((Number (String.sub text i (!j - i)), i) :: acc)
      | c when alpha c ->
          let j = word_end i identifier_extra in
          scan j ((Ident (String.sub text i (j - i)), i) :: acc)
      | c when Char.code c < 128 && c <> '\000' ->
          scan (i + 1) ((Symbol c, i) :: acc)
      | _ -> raise (Error (Printf.sprintf "Unexpected character at byte %d" i))
  in
  scan 0 []

let create ?keywords ?identifier_extra text =
  { tokens = tokenize ?keywords ?identifier_extra text; pos = 0 }

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
  if peek p = Keyword word then (
    ignore (next p);
    true)
  else false

let expect_keyword p word =
  if not (eat_keyword p word) then error p ("Expected \\" ^ word)

let identifier p =
  match peek p with
  | Ident s ->
      ignore (next p);
      s
  | _ -> error p "Expected identifier"

let keyword p =
  match peek p with
  | Keyword s ->
      ignore (next p);
      s
  | _ -> error p "Expected keyword"

let character p =
  match peek p with
  | Character s ->
      ignore (next p);
      s
  | _ -> error p "Expected character literal"

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

let parse f text =
  protect (fun () ->
      let p = create text in
      let value = f p in
      expect_eof p;
      value)

let rec separated p ~separator item =
  let first = item p in
  if eat_symbol p separator then first :: separated p ~separator item
  else [ first ]
