open Syntax
module P = Utils_parsing.Parser

type t = code

let reg p =
  P.expect_symbol p '%';
  P.identifier p

let value p =
  if P.peek p = P.Symbol '%' then Reg (reg p)
  else if P.bump_if_symbol p ':' then Label (P.identifier p)
  else
    match P.peek p with
    | P.Ident name when String.starts_with ~prefix:"MEM_" name ->
        ignore (P.bump p);
        let suffix = String.sub name 4 (String.length name - 4) in
        let n = Utils_base.Error.get (N.of_string suffix) in
        Memory (Utils_base.Error.get (N.to_int n))
    | _ ->
        ignore (P.bump_if_symbol p '#');
        Imm (P.natural p)

let place p =
  P.expect_keyword p "deref";
  Deref (value p)

let target p =
  match value p with
  | (Reg _ | Label _) as v -> v
  | _ -> P.error p "Jump target must be a register or code label"

let rhs p =
  if P.bump_if_keyword p "ld" then Load (place p)
  else if P.bump_if_keyword p "slice" then
    let s = value p in
    let b = value p in
    let e = value p in
    Slice (s, b, e)
  else if P.bump_if_keyword p "addr" then
    let s = value p in
    let i = value p in
    Addr (s, i)
  else if P.bump_if_keyword p "len" then Len (value p)
  else
    let a = value p in
    if P.bump_if_symbol p '+' then Binop (a, Add, value p)
    else if P.bump_if_symbol p '-' then Binop (a, Sub, value p)
    else Value a

let stmt p =
  let s =
    if P.bump_if_keyword p "print" then Print (value p)
    else if P.bump_if_keyword p "input" then Input (place p)
    else if P.peek p = P.Symbol '%' then (
      let r = reg p in
      P.expect p ":=";
      Assign (r, rhs p))
    else
      let dst = place p in
      P.expect p ":= \\st";
      Store (dst, value p)
  in
  P.expect_symbol p ';';
  s

let cond p =
  P.binary p
    ~operators:[ ("==", Eq); ("<", Lt); (">", Gt) ]
    ~operand:value
    ~make:(fun a op b -> (a, op, b))

let block p =
  P.expect_symbol p ':';
  let label = P.identifier p in
  P.group p P.Braces (fun p ->
      let rec body stmts ifs =
        if P.bump_if_keyword p "goto" then finish stmts ifs (Goto (target p))
        else if P.bump_if_keyword p "halt" then finish stmts ifs Halt
        else if P.bump_if_keyword p "if" then (
          let cond = cond p in
          P.expect_keyword p "then";
          let target = target p in
          P.expect_symbol p ';';
          body stmts ({ cond; target } :: ifs))
        else if P.eof p then P.error p "Missing terminator"
        else if ifs <> [] then P.error p "Statement after conditional jump"
        else
          let st = stmt p in
          body (st :: stmts) ifs
      and finish stmts ifs ending =
        P.expect_symbol p ';';
        P.expect_eof p;
        { label; stmts = List.rev stmts; ifs = List.rev ifs; ending }
      in
      body [] [])

let parse =
  P.parse (fun p ->
      P.expect_keyword p "memories";
      let memories = P.integer p in
      P.expect_symbol p ';';
      if memories < 1 then P.error p "At least one memory is required";
      let blocks = P.many p block in
      { memories; blocks })

let value_text = function
  | Reg r -> "%" ^ Id.to_string r
  | Imm n -> N.to_string n
  | Label l -> ":" ^ Id.to_string l
  | Memory i -> "MEM_" ^ string_of_int i

let place_text (Deref v) = "\\deref " ^ value_text v

let rhs_text = function
  | Value v -> value_text v
  | Binop (a, op, b) ->
      value_text a ^ (if op = Add then " + " else " - ") ^ value_text b
  | Load p -> "\\ld " ^ place_text p
  | Slice (s, b, e) ->
      "\\slice " ^ String.concat " " (List.map value_text [ s; b; e ])
  | Addr (s, i) -> "\\addr " ^ value_text s ^ " " ^ value_text i
  | Len s -> "\\len " ^ value_text s

let stmt_text = function
  | Print v -> "\\print " ^ value_text v ^ ";"
  | Input p -> "\\input " ^ place_text p ^ ";"
  | Assign (r, e) -> "%" ^ Id.to_string r ^ " := " ^ rhs_text e ^ ";"
  | Store (p, v) -> place_text p ^ " := \\st " ^ value_text v ^ ";"

let cond_text (a, op, b) =
  value_text a
  ^ (match op with Eq -> " == " | Lt -> " < " | Gt -> " > ")
  ^ value_text b

let print c =
  "\\memories " ^ string_of_int c.memories ^ ";\n"
  ^ String.concat ""
      (List.map
         (fun b ->
           ":" ^ Id.to_string b.label ^ " {\n"
           ^ String.concat ""
               (List.map (fun s -> "  " ^ stmt_text s ^ "\n") b.stmts)
           ^ String.concat ""
               (List.map
                  (fun j ->
                    "  \\if " ^ cond_text j.cond ^ " \\then "
                    ^ value_text j.target ^ ";\n")
                  b.ifs)
           ^ "  "
           ^ (match b.ending with
             | Halt -> "\\halt;"
             | Goto v -> "\\goto " ^ value_text v ^ ";")
           ^ "\n}\n")
         c.blocks)
