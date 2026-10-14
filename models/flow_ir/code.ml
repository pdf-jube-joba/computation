open Syntax

type t = code

let rec value_to_text = function
  | VReg r -> "%" ^ Utils_names.Identifier.to_string r
  | Imm n -> "#" ^ N.to_string n
  | CodeLabel l -> ":" ^ Utils_names.Identifier.to_string l
  | Ref p -> "ref " ^ place_to_text p

and place_to_text = function
  | Label l -> "@" ^ Utils_names.Identifier.to_string l
  | Deref r -> "deref %" ^ Utils_names.Identifier.to_string r
  | SAcc v -> "sacc " ^ value_to_text v
  | HAcc (h, i) -> "hacc(" ^ value_to_text h ^ ")[" ^ value_to_text i ^ "]"

let cond_to_text c =
  let a, op, b =
    match c with
    | Lt (a, b) -> (a, "<", b)
    | Eq (a, b) -> (a, "=", b)
    | Gt (a, b) -> (a, ">", b)
  in
  value_to_text a ^ " " ^ op ^ " " ^ value_to_text b

let stmt_to_text = function
  | Nop -> "nop;"
  | Load (r, p) ->
      "%"
      ^ Utils_names.Identifier.to_string r
      ^ " := ld " ^ place_to_text p ^ ";"
  | Assign (r, v) ->
      "%" ^ Utils_names.Identifier.to_string r ^ " := " ^ value_to_text v ^ ";"
  | BinOp (r, a, op, b) ->
      "%"
      ^ Utils_names.Identifier.to_string r
      ^ " := " ^ value_to_text a
      ^ (if op = Add then " + " else " - ")
      ^ value_to_text b ^ ";"
  | Store (p, v) -> place_to_text p ^ " := st " ^ value_to_text v ^ ";"
  | Pop r -> "pop %" ^ Utils_names.Identifier.to_string r ^ ";"
  | Push v -> "push " ^ value_to_text v ^ ";"
  | LGet r -> "lget %" ^ Utils_names.Identifier.to_string r ^ ";"
  | HAlloc (v, r) ->
      "halloc(" ^ value_to_text v ^ ") %"
      ^ Utils_names.Identifier.to_string r
      ^ ";"
  | HFree v -> "hfree " ^ value_to_text v ^ ";"
  | Print r -> "print %" ^ Utils_names.Identifier.to_string r ^ ";"
  | Input p -> "input " ^ place_to_text p ^ ";"

let print (p : code) =
  let b = Buffer.create 512 in
  let line s = Buffer.add_string b (s ^ "\n") in
  List.iter
    (fun (s : static_def) ->
      line
        ("@"
        ^ Utils_names.Identifier.to_string s.label
        ^ " " ^ N.to_string s.value))
    p.statics;
  List.iter
    (fun (r : region) ->
      line (":" ^ Utils_names.Identifier.to_string r.label ^ " {");
      List.iter
        (fun (bb : block) ->
          line ("  :" ^ Utils_names.Identifier.to_string bb.label ^ " {");
          List.iter (fun s -> line ("    " ^ stmt_to_text s)) bb.stmts;
          List.iter
            (fun j ->
              line
                ("    if " ^ cond_to_text j.cond ^ " then "
               ^ value_to_text j.target ^ ";"))
            (ifs bb.cont);
          line
            ("    "
            ^
            match bb.cont with
            | Go (_, v) -> "goto " ^ value_to_text v ^ ";"
            | Enter (_, v) -> "enter " ^ value_to_text v ^ ";"
            | Halt _ -> "halt;");
          line "  }")
        r.blocks;
      line "}")
    p.regions;
  Buffer.contents b

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
