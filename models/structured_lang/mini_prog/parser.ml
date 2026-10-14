open Structured_lang_shared.Arithmetic
open Syntax

module Code = struct
  include Print

  let parse f text = L.parse ~character_literals:true f text

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
    if L.peek p = L.Symbol '[' then
      L.group p ~opening:'[' (fun p ->
          let t = parse_type p in
          L.expect_symbol p ';';
          let len = L.integer p in
          Array (t, len))
    else
      let t = type_atom p in
      let rec loop kind acc =
        if L.bump_if_symbol p '*' then (
          if kind = Some `Sum then
            L.error p "cannot mix '*' and '+' in a type without brackets";
          let t = type_atom p in
          loop (Some `Product) (t :: acc))
        else if L.bump_if_symbol p '+' then (
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

  let terminator p = L.eof p || L.peek p = L.Symbol ';'

  let stmt_start p =
    match L.peek p with
    | L.Ident
        ( "assign" | "if" | "case" | "halloc" | "hfree" | "loop" | "break"
        | "continue" | "call" | "return" | "block" ) ->
        true
    | _ -> false

  let rec parse_place p =
    let base =
      if L.bump_if_keyword p "static" then Static (L.identifier p)
      else if L.bump_if_keyword p "local" then Local (L.identifier p)
      else
        let v = L.group p ~opening:'(' (fun p -> parse_value p L.eof) in
        L.expect p "#loc";
        Deref v
    in
    let rec project pbase =
      if L.bump_if_symbol p '.' then
        let i = L.integer p in
        project (Field (pbase, i))
      else if L.bump_if_symbol p '?' then
        let i = L.integer p in
        project (Tag_place (pbase, i))
      else if L.peek p = L.Symbol '[' then
        let i = L.group p ~opening:'[' (fun p -> parse_value p L.eof) in
        project (Index (pbase, i))
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
        | L.Number _ -> push (Number (L.natural p))
        | L.Character_lit c ->
            ignore (L.bump p);
            push (Char c)
        | _ when L.bump_if p "#true" -> push (Boolean true)
        | _ when L.bump_if p "#false" -> push (Boolean false)
        | _ when L.bump_if p "#unit" -> push Unit_value
        | _ when L.bump_if p "#null_ptr" -> push NullPtr
        | _ when L.bump_if p "#null_fn" -> push NullFn
        | _ when L.bump_if_keyword p "fn" -> push (Function (L.identifier p))
        | _ when L.bump_if_keyword p "ld" -> push (Load (parse_place p))
        | L.Ident ("static" | "local") | L.Symbol '(' ->
            let place = parse_place p in
            L.expect p "#addr";
            push (Addr place)
        | _ when L.bump_if_symbol p '!' -> unary (fun v -> UnOp (Not, v))
        | _ when L.bump_if_symbol p '+' -> binary Add
        | _ when L.bump_if_symbol p '-' -> binary Sub
        | _ when L.bump_if p "==" -> binary Eq
        | _ when L.bump_if_symbol p '<' -> binary Lt
        | _ when L.bump_if p "&&" -> binary And
        | _ when L.bump_if p "||" -> binary Or
        | _ when L.bump_if_keyword p "pair" ->
            let count = L.group p ~opening:'(' L.integer in
            let rec take n xs acc =
              if n = 0 then (acc, xs)
              else
                match xs with
                | x :: xs -> take (n - 1) xs (x :: acc)
                | _ -> L.error p "pair arity exceeds expression stack"
            in
            let items, ss = take count stack [] in
            loop (Pair items :: ss)
        | _ when L.bump_if_keyword p "tag" ->
            let tag = L.group p ~opening:'(' L.integer in
            unary (fun v -> Tag (tag, v))
        | _ -> L.error p "Unexpected token in value expression"
    in
    loop []

  let value_list p stop =
    if L.eof p || stop p then []
    else
      L.separated p ~separator:',' (fun p ->
          parse_value p (fun p -> L.peek p = L.Symbol ',' || stop p))

  let rec parse_stmt p =
    if L.bump_if_keyword p "assign" then (
      let place = parse_place p in
      L.expect p ":=";
      Assign (place, parse_value p terminator))
    else if L.bump_if_keyword p "if" then
      let c = parse_value p stmt_start in
      let s = parse_stmt p in
      If (c, s)
    else if L.bump_if_keyword p "case" then (
      let tag = L.integer p in
      L.expect_keyword p "of";
      let v = parse_value p stmt_start in
      let s = parse_stmt p in
      Case (tag, v, s))
    else if L.bump_if_keyword p "halloc" then (
      let ty = parse_type p in
      L.expect p "->";
      let place = parse_place p in
      HAlloc (ty, place))
    else if L.bump_if_keyword p "hfree" then HFree (parse_value p terminator)
    else if L.bump_if_keyword p "loop" then (
      let l = L.identifier p in
      L.expect_symbol p ':';
      Loop (l, parse_stmt p))
    else if L.bump_if_keyword p "break" then Break (L.identifier p)
    else if L.bump_if_keyword p "continue" then Continue (L.identifier p)
    else if L.bump_if_keyword p "call" then (
      let f = parse_value p (fun p -> L.peek p = L.Symbol '(') in
      let args = L.group p ~opening:'(' (fun p -> value_list p L.eof) in
      L.expect p "->";
      let places = L.separated p ~separator:',' parse_place in
      Call (f, args, places))
    else if L.bump_if_keyword p "return" then Return (value_list p terminator)
    else if L.bump_if_keyword p "block" then Block (parse_block p)
    else L.error p "Unexpected token in statement"

  and parse_block p =
    let bindings =
      L.group p ~opening:'(' (fun p ->
          L.separated_list p ~separator:',' (fun p ->
              let n = L.identifier p in
              L.expect_symbol p ':';
              let ty = parse_type p in
              (n, ty)))
    in
    let stmts =
      L.group p ~opening:'{' (fun p ->
          L.separated_list ~allow_trailing:true p ~separator:';' parse_stmt)
    in
    { bindings; stmts }

  let parse =
    parse (fun p ->
        let rec loop ss fs =
          if L.eof p then { statics = List.rev ss; functions = List.rev fs }
          else if L.bump_if_keyword p "static" then (
            let name = L.identifier p in
            L.expect_symbol p ':';
            let ty = parse_type p in
            loop ({ name; ty } :: ss) fs)
          else (
            L.expect_keyword p "fn";
            let name = L.identifier p in
            let block = parse_block p in
            loop ss ({ name; block } :: fs))
        in
        loop [] [])
end

module Ainput = Utils_codec.Text.Natural_list
