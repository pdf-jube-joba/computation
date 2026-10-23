open Syntax
module L = Utils_parsing.Parser

let identifier _statements p = L.identifier p
let number = L.natural

let rec expression statements p =
  if L.peek p = L.Keyword "if" then if_expr statements p
  else binary statements p

and if_expr statements p =
  L.expect_keyword p "if";
  let c = expression statements p in
  L.expect_keyword p "then";
  let t = expression statements p in
  L.expect_keyword p "else";
  let e = expression statements p in
  L.expect_keyword p "fi";
  If (c, t, e)

and binary statements p =
  L.expression
    ~infix:
      [
        ("+", 1, 2, fun a b -> BinOp (a, Add, b));
        ("-", 1, 2, fun a b -> BinOp (a, Sub, b));
        ("&&", 1, 2, fun a b -> BinOp (a, And, b));
      ]
    ~atom:(fun _ p -> application statements p)
    p

and application statements p =
  let rec loop f =
    if L.is_group p L.Parentheses then
      let x = L.group p L.Parentheses (expression statements) in
      loop (App (f, x))
    else f
  in
  loop (prefix statements p)

and prefix statements p =
  if L.bump_if_keyword p "inc" then UnOp (Inc, prefix statements p)
  else if L.bump_if_keyword p "dec" then UnOp (Dec, prefix statements p)
  else if L.bump_if_keyword p "not" then UnOp (Not, prefix statements p)
  else atom statements p

and atom statements p =
  match L.peek p with
  | L.Number _ -> Nat (number p)
  | L.Keyword "true" ->
      ignore (L.bump p);
      Bool true
  | L.Keyword "false" ->
      ignore (L.bump p);
      Bool false
  | L.Keyword "unit" ->
      ignore (L.bump p);
      Unit
  | L.Keyword "print" when not statements ->
      ignore (L.bump p);
      Print (number p)
  | L.Keyword "fun" ->
      ignore (L.bump p);
      let x = identifier statements p in
      L.expect p "=>";
      Fun (x, expression statements p)
  | L.Keyword "rec" ->
      ignore (L.bump p);
      let f = identifier statements p in
      let x = identifier statements p in
      L.expect p "=>";
      Rec (f, x, expression statements p)
  | L.Keyword "if" -> if_expr statements p
  | _ when L.is_group p L.Parentheses ->
      L.group p L.Parentheses (expression statements)
  | _ when statements && L.is_group p L.Braces ->
      let s, e = L.group p L.Braces block_contents in
      Block (s, e)
  | _ -> Var (identifier statements p)

and simple_stmt p =
  if L.bump_if_keyword p "skip" then Skip
  else if L.bump_if_keyword p "print" then PrintStmt (number p)
  else if L.bump_if_keyword p "let" then (
    let x = identifier true p in
    L.expect p ":=";
    Let (x, expression true p))
  else if L.bump_if_keyword p "if" then (
    let c = expression true p in
    L.expect_keyword p "then";
    let s = statement p in
    L.expect_keyword p "end";
    IfStmt (c, s))
  else if L.bump_if_keyword p "while" then (
    let c = expression true p in
    L.expect_keyword p "do";
    let s = statement p in
    L.expect_keyword p "end";
    While (c, s))
  else L.error p "statement expected"

and statement p =
  L.chain_left p ~separator:';' ~combine:(fun a b -> Seq (a, b)) simple_stmt

and block_contents p =
  let s = simple_stmt p in
  L.expect_symbol p ';';
  match L.attempt p block_contents with
  | Ok (t, e) -> (Seq (s, t), e)
  | Error _ -> (s, expression true p)

let parse = L.parse (expression false)
