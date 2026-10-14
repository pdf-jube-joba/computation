open Syntax
module L = Utils_parsing.Parser

let vreg p =
  L.expect_symbol p '%';
  L.identifier p

let rec parse_value p =
  if L.bump_if_keyword p "ref" then Ref (parse_place p)
  else if L.peek p = L.Symbol '%' then VReg (vreg p)
  else if L.bump_if_symbol p ':' then CodeLabel (L.identifier p)
  else (
    ignore (L.bump_if_symbol p '#');
    Imm (L.natural p))

and parse_place p =
  if L.bump_if_symbol p '@' then Label (L.identifier p)
  else if L.bump_if_keyword p "deref" then Deref (vreg p)
  else if L.bump_if_keyword p "sacc" then SAcc (parse_value p)
  else if L.bump_if_keyword p "hacc" then
    let h = L.group p ~opening:'(' parse_value in
    let i = L.group p ~opening:'[' parse_value in
    HAcc (h, i)
  else L.error p "Expected place expression"

let parse_cond p =
  L.binary p
    ~operators:
      [
        ("<", fun a b -> Lt (a, b));
        ("=", fun a b -> Eq (a, b));
        (">", fun a b -> Gt (a, b));
      ]
    ~operand:parse_value
    ~make:(fun a op b -> op a b)

let parse_stmt p =
  let s =
    if L.bump_if_keyword p "nop" then Nop
    else if L.bump_if_keyword p "pop" then Pop (vreg p)
    else if L.bump_if_keyword p "push" then Push (parse_value p)
    else if L.bump_if_keyword p "lget" then LGet (vreg p)
    else if L.bump_if_keyword p "hfree" then HFree (parse_value p)
    else if L.bump_if_keyword p "print" then Print (vreg p)
    else if L.bump_if_keyword p "input" then Input (parse_place p)
    else if L.bump_if_keyword p "halloc" then
      let v = L.group p ~opening:'(' parse_value in
      let r = vreg p in
      HAlloc (v, r)
    else if L.peek p = L.Symbol '%' then (
      let r = vreg p in
      L.expect p ":=";
      if L.bump_if_keyword p "ld" then Load (r, parse_place p)
      else
        let a = parse_value p in
        if L.bump_if_symbol p '+' then BinOp (r, a, Add, parse_value p)
        else if L.bump_if_symbol p '-' then BinOp (r, a, Sub, parse_value p)
        else Assign (r, a))
    else
      let place = parse_place p in
      L.expect p ":= st";
      Store (place, parse_value p)
  in
  L.expect_symbol p ';';
  s

let parse_block p =
  L.expect_symbol p ':';
  let label = L.identifier p in
  L.group p ~opening:'{' (fun p ->
      let rec loop stmts jumps cont =
        if L.eof p then
          {
            label;
            stmts = List.rev stmts;
            cont =
              (match cont with
              | Some c -> c
              | None -> L.error p "Missing terminator");
          }
        else if L.bump_if_keyword p "if" then (
          let cond = parse_cond p in
          L.expect_keyword p "then";
          let target = parse_value p in
          L.expect_symbol p ';';
          loop stmts (jumps @ [ { cond; target } ]) cont)
        else if cont <> None then L.error p "Line exists after terminator"
        else if L.bump_if_keyword p "goto" then (
          let v = parse_value p in
          L.expect_symbol p ';';
          loop stmts jumps (Some (Go (jumps, v))))
        else if L.bump_if_keyword p "enter" then (
          let v = parse_value p in
          L.expect_symbol p ';';
          loop stmts jumps (Some (Enter (jumps, v))))
        else if L.bump_if_keyword p "halt" then (
          L.expect_symbol p ';';
          loop stmts jumps (Some (Halt jumps)))
        else loop (parse_stmt p :: stmts) jumps cont
      in
      loop [] [] None)

let parse =
  L.parse (fun p ->
      let rec loop ss rs =
        if L.eof p then { statics = List.rev ss; regions = List.rev rs }
        else if L.bump_if_symbol p '@' then
          let label = L.identifier p in
          let value = L.natural p in
          loop ({ label; value } :: ss) rs
        else (
          L.expect_symbol p ':';
          let label = L.identifier p in
          let blocks = L.group p ~opening:'{' (fun p -> L.many p parse_block) in
          if blocks = [] then L.error p "Region has no blocks";
          loop ss ({ label; blocks } :: rs))
      in
      loop [] [])
