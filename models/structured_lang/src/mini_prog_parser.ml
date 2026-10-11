open Common
open Mini_prog_syntax

(* Single Unicode scalar character literals are carried as String_lit tokens;
   all other tokens and comments use the shared lexer. *)
let create text =
  let tokens = ref [] in
  let n = String.length text in
  let rec scan start i =
    if i >= n then
      let p = L.create (String.sub text start (n - start)) in
      Array.iter
        (fun (t, k) -> if t <> L.Eof then tokens := (t, k + start) :: !tokens)
        p.tokens
    else if i + 1 < n && text.[i] = '/' && text.[i + 1] = '/' then (
      let j = ref (i + 2) in
      while !j < n && text.[!j] <> '\n' do
        incr j
      done;
      scan start !j)
    else if text.[i] = '\'' then (
      let j = ref (i + 1) in
      while !j < n && text.[!j] <> '\'' do
        incr j
      done;
      if !j = n then failwith "invalid char literal";
      let c = String.sub text (i + 1) (!j - i - 1) in
      let len =
        if c = "" then 0
        else
          let b = Char.code c.[0] in
          if b < 128 then 1 else if b < 224 then 2 else if b < 240 then 3 else 4
      in
      if String.length c <> len || not (Flow_ir.valid_utf8 c) then
        failwith "invalid char literal";
      let p = L.create (String.sub text start (i - start)) in
      Array.iter
        (fun (t, k) -> if t <> L.Eof then tokens := (t, k + start) :: !tokens)
        p.tokens;
      tokens := (L.String_lit c, i) :: !tokens;
      scan (!j + 1) (!j + 1))
    else scan start (i + 1)
  in
  scan 0 0;
  { L.tokens = Array.of_list (List.rev ((L.Eof, n) :: !tokens)); pos = 0 }

let parse f text = L.parse_with create f text

let type_atom p =
  L.expect_symbol p '#';
  match L.identifier p with
  | "num" -> Num
  | "u8" -> U8
  | "bool" -> Bool
  | "unit" -> Unit
  | "ptr" -> Ptr
  | "fn" -> Fn
  | _ -> L.error p "Expected type atom"

let rec parse_type p =
  if L.eat_symbol p '[' then (
    let t = parse_type p in
    L.expect_symbol p ';';
    let len = L.integer p in
    L.expect_symbol p ']';
    Array (t, len))
  else
    let t = type_atom p in
    let rec loop kind acc =
      if L.eat_symbol p '*' then (
        if kind = Some `Sum then
          L.error p "cannot mix '*' and '+' in a type without brackets";
        let t = type_atom p in
        loop (Some `Product) (t :: acc))
      else if L.eat_symbol p '+' then (
        if kind = Some `Product then
          L.error p "cannot mix '*' and '+' in a type without brackets";
        let t = type_atom p in
        loop (Some `Sum) (t :: acc))
      else
        match kind with
        | None -> t
        | Some `Product -> Product (List.rev acc)
        | Some `Sum -> Sum (List.rev acc)
    in
    loop None [ t ]

let terminator p =
  match L.peek p with L.Symbol ';' | L.Symbol '}' -> true | _ -> false

let stmt_start p =
  match L.peek p with
  | L.Ident
      ( "assign" | "if" | "case" | "halloc" | "hfree" | "loop" | "break"
      | "continue" | "call" | "return" | "block" ) ->
      true
  | _ -> false

let rec parse_place p =
  let base =
    if L.eat_keyword p "static" then Static (ident p)
    else if L.eat_keyword p "local" then Local (ident p)
    else (
      L.expect_symbol p '(';
      let v = parse_value p (fun p -> L.peek p = L.Symbol ')') in
      L.expect p ") #loc";
      Deref v)
  in
  let rec project pbase =
    if L.eat_symbol p '.' then
      let i = L.integer p in
      project (Field (pbase, i))
    else if L.eat_symbol p '?' then
      let i = L.integer p in
      project (Tag_place (pbase, i))
    else if L.eat_symbol p '[' then (
      let i = parse_value p (fun p -> L.peek p = L.Symbol ']') in
      L.expect_symbol p ']';
      project (Index (pbase, i)))
    else pbase
  in
  project base

and parse_value p stop =
  let rec loop stack =
    if L.eof p || stop p then
      match stack with
      | [ v ] -> v
      | _ -> L.error p "value expression did not reduce to a single value"
    else
      let push v = loop (v :: stack) in
      let unary f =
        match stack with
        | v :: ss -> loop (f v :: ss)
        | _ -> L.error p "operator needs one operand"
      in
      let binary op =
        match stack with
        | b :: a :: ss -> loop (BinOp (a, op, b) :: ss)
        | _ -> L.error p "operator needs two operands"
      in
      match L.peek p with
      | L.Number _ -> push (Number (number p))
      | L.String_lit c ->
          ignore (L.next p);
          push (Char c)
      | _ when L.eat p "#true" -> push (Boolean true)
      | _ when L.eat p "#false" -> push (Boolean false)
      | _ when L.eat p "#unit" -> push Unit_value
      | _ when L.eat p "#null_ptr" -> push NullPtr
      | _ when L.eat p "#null_fn" -> push NullFn
      | _ when L.eat_keyword p "fn" -> push (Function (ident p))
      | _ when L.eat_keyword p "ld" -> push (Load (parse_place p))
      | L.Ident ("static" | "local") | L.Symbol '(' ->
          let place = parse_place p in
          L.expect p "#addr";
          push (Addr place)
      | _ when L.eat_symbol p '!' -> unary (fun v -> UnOp (Not, v))
      | _ when L.eat_symbol p '+' -> binary Add
      | _ when L.eat_symbol p '-' -> binary Sub
      | _ when L.eat p "==" -> binary Eq
      | _ when L.eat_symbol p '<' -> binary Lt
      | _ when L.eat p "&&" -> binary And
      | _ when L.eat p "||" -> binary Or
      | _ when L.eat_keyword p "pair" ->
          L.expect_symbol p '(';
          let count = L.integer p in
          L.expect_symbol p ')';
          let rec take n xs acc =
            if n = 0 then (acc, xs)
            else
              match xs with
              | x :: xs -> take (n - 1) xs (x :: acc)
              | _ -> L.error p "pair arity exceeds expression stack"
          in
          let items, ss = take count stack [] in
          loop (Pair items :: ss)
      | _ when L.eat_keyword p "tag" ->
          L.expect_symbol p '(';
          let tag = L.integer p in
          L.expect_symbol p ')';
          unary (fun v -> Tag (tag, v))
      | _ -> L.error p "Unexpected token in value expression"
  in
  loop []

let value_list p stop =
  if stop p then []
  else
    let rec loop acc =
      let v = parse_value p (fun p -> L.peek p = L.Symbol ',' || stop p) in
      if L.eat_symbol p ',' then loop (v :: acc) else List.rev (v :: acc)
    in
    loop []

let rec parse_stmt p =
  if L.eat_keyword p "assign" then (
    let place = parse_place p in
    L.expect p ":=";
    Assign (place, parse_value p terminator))
  else if L.eat_keyword p "if" then
    let c = parse_value p stmt_start in
    let s = parse_stmt p in
    If (c, s)
  else if L.eat_keyword p "case" then (
    let tag = L.integer p in
    L.expect_keyword p "of";
    let v = parse_value p stmt_start in
    let s = parse_stmt p in
    Case (tag, v, s))
  else if L.eat_keyword p "halloc" then (
    let ty = parse_type p in
    L.expect p "->";
    let place = parse_place p in
    HAlloc (ty, place))
  else if L.eat_keyword p "hfree" then HFree (parse_value p terminator)
  else if L.eat_keyword p "loop" then (
    let l = ident p in
    L.expect_symbol p ':';
    Loop (l, parse_stmt p))
  else if L.eat_keyword p "break" then Break (ident p)
  else if L.eat_keyword p "continue" then Continue (ident p)
  else if L.eat_keyword p "call" then (
    let f = parse_value p (fun p -> L.peek p = L.Symbol '(') in
    L.expect_symbol p '(';
    let args = value_list p (fun p -> L.peek p = L.Symbol ')') in
    L.expect p ") ->";
    let rec places acc =
      let v = parse_place p in
      if L.eat_symbol p ',' then places (v :: acc) else List.rev (v :: acc)
    in
    Call (f, args, places []))
  else if L.eat_keyword p "return" then Return (value_list p terminator)
  else if L.eat_keyword p "block" then Block (parse_block p)
  else L.error p "Unexpected token in statement"

and parse_block p =
  L.expect_symbol p '(';
  let rec bindings acc =
    let n = ident p in
    L.expect_symbol p ':';
    let ty = parse_type p in
    if L.eat_symbol p ',' then bindings ((n, ty) :: acc)
    else List.rev ((n, ty) :: acc)
  in
  let bindings = if L.peek p = L.Symbol ')' then [] else bindings [] in
  L.expect p ") {";
  let rec stmts acc =
    if L.eat_symbol p '}' then List.rev acc
    else
      let s = parse_stmt p in
      if (not (L.eat_symbol p ';')) && L.peek p <> L.Symbol '}' then
        L.error p "expected ';' or '}' after statement";
      stmts (s :: acc)
  in
  let stmts = stmts [] in
  { bindings; stmts }

let parse_code =
  parse (fun p ->
      let rec loop ss fs =
        if L.eof p then { statics = List.rev ss; functions = List.rev fs }
        else if L.eat_keyword p "static" then (
          let name = ident p in
          L.expect_symbol p ':';
          let ty = parse_type p in
          loop ({ name; ty } :: ss) fs)
        else (
          L.expect_keyword p "fn";
          let name = ident p in
          let block = parse_block p in
          loop ss ({ name; block } :: fs))
      in
      loop [] [])

let parse_ainput text =
  L.protect (fun () ->
      let s = String.trim text in
      if String.length s < 2 || s.[0] <> '(' || s.[String.length s - 1] <> ')'
      then failwith "Expected to start with '(' and end with ')'";
      String.sub s 1 (String.length s - 2)
      |> String.split_on_char ','
      |> List.filter (fun s -> String.trim s <> "")
      |> List.map (fun s -> get (N.of_string s)))
