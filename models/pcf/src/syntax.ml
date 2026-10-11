module N = Utils.Number
module L = Utils.Lexer
module Names = Set.Make (String)

type binop = Add | Sub | And [@@deriving yojson]
type unop = Inc | Dec | Not [@@deriving yojson]

(* Blocks are the statement-language extension; the expression parser rejects them. *)
type expr =
  | Nat of N.t
  | Bool of bool
  | Unit
  | Print of N.t
  | Var of Utils.Identifier.t
  | BinOp of expr * binop * expr
  | UnOp of unop * expr
  | Fun of Utils.Identifier.t * expr
  | Rec of Utils.Identifier.t * Utils.Identifier.t * expr
  | App of expr * expr
  | If of expr * expr * expr
  | Block of stmt * expr

and stmt =
  | Skip
  | PrintStmt of N.t
  | Seq of stmt * stmt
  | Let of Utils.Identifier.t * expr
  | IfStmt of expr * stmt
  | While of expr * stmt
[@@deriving yojson]

type print_effect = N.t option [@@deriving yojson]

let print_binop = function Add -> "+" | Sub -> "-" | And -> "&&"
let print_unop = function Inc -> "inc" | Dec -> "dec" | Not -> "not"

let rec print = function
  | Nat n -> N.to_string n
  | Bool true -> "#true"
  | Bool false -> "#false"
  | Unit -> "#unit"
  | Print n -> "print " ^ N.to_string n
  | Var x -> x
  | BinOp (a, o, b) -> "(" ^ print a ^ " " ^ print_binop o ^ " " ^ print b ^ ")"
  | UnOp (o, e) -> "(" ^ print_unop o ^ " " ^ print e ^ ")"
  | Fun (x, b) -> "(fun " ^ x ^ " => " ^ print b ^ ")"
  | Rec (f, x, b) -> "(rec " ^ f ^ " " ^ x ^ " => " ^ print b ^ ")"
  | App (f, x) -> print f ^ "(" ^ print x ^ ")"
  | If (c, t, e) ->
      "(if " ^ print c ^ " then " ^ print t ^ " else " ^ print e ^ " fi)"
  | Block (s, e) -> "{ " ^ print_stmt s ^ "; " ^ print e ^ " }"

and print_stmt = function
  | Skip -> "skip"
  | PrintStmt n -> "print " ^ N.to_string n
  | Seq (a, b) -> print_stmt a ^ "; " ^ print_stmt b
  | Let (x, e) -> "let " ^ x ^ " := " ^ print e
  | IfStmt (c, s) -> "if " ^ print c ^ " then " ^ print_stmt s ^ " end"
  | While (c, s) -> "while " ^ print c ^ " do " ^ print_stmt s ^ " end"

let is_value = function
  | Nat _ | Bool _ | Unit | Fun _ | Rec _ -> true
  | _ -> false

let expect_nat = function
  | Nat n -> Ok n
  | _ -> Error "final value is not a natural number"

let apply_inputs e inputs = List.fold_left (fun e n -> App (e, Nat n)) e inputs

let reserved statements x =
  List.mem x
    ([ "print"; "fun"; "rec"; "if"; "then"; "else"; "fi"; "inc"; "dec"; "not" ]
    @ if statements then [ "skip"; "let"; "while"; "do"; "end" ] else [])

let identifier statements p =
  let x = L.identifier p in
  if reserved statements x then
    L.error p ("identifier expected, found keyword '" ^ x ^ "'");
  x

let number p =
  match N.of_string (L.natural p) with Ok n -> n | Error e -> L.error p e

let rec expression statements p =
  if L.peek p = L.Ident "if" then if_expr statements p else binary statements p

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
  let rec loop a =
    let op =
      if L.eat_symbol p '+' then Some Add
      else if L.eat_symbol p '-' then Some Sub
      else if L.eat p "&&" then Some And
      else None
    in
    match op with
    | None -> a
    | Some op ->
        let b = application statements p in
        loop (BinOp (a, op, b))
  in
  loop (application statements p)

and application statements p =
  let rec loop f =
    if L.eat_symbol p '(' then (
      let x = expression statements p in
      L.expect_symbol p ')';
      loop (App (f, x)))
    else f
  in
  loop (prefix statements p)

and prefix statements p =
  if L.eat_keyword p "inc" then UnOp (Inc, prefix statements p)
  else if L.eat_keyword p "dec" then UnOp (Dec, prefix statements p)
  else if L.eat_keyword p "not" then UnOp (Not, prefix statements p)
  else atom statements p

and atom statements p =
  match L.peek p with
  | L.Number _ -> Nat (number p)
  | L.Symbol '#' -> (
      ignore (L.next p);
      match L.identifier p with
      | "true" -> Bool true
      | "false" -> Bool false
      | "unit" -> Unit
      | k -> L.error p ("unknown token: #" ^ k))
  | L.Ident "print" when not statements ->
      ignore (L.next p);
      Print (number p)
  | L.Ident "fun" ->
      ignore (L.next p);
      let x = identifier statements p in
      L.expect p "=>";
      Fun (x, expression statements p)
  | L.Ident "rec" ->
      ignore (L.next p);
      let f = identifier statements p in
      let x = identifier statements p in
      L.expect p "=>";
      Rec (f, x, expression statements p)
  | L.Ident "if" -> if_expr statements p
  | L.Symbol '(' ->
      ignore (L.next p);
      let e = expression statements p in
      L.expect_symbol p ')';
      e
  | L.Symbol '{' when statements ->
      ignore (L.next p);
      let s, e = block_contents p in
      L.expect_symbol p '}';
      Block (s, e)
  | _ -> Var (identifier statements p)

and simple_stmt p =
  if L.eat_keyword p "skip" then Skip
  else if L.eat_keyword p "print" then PrintStmt (number p)
  else if L.eat_keyword p "let" then (
    let x = identifier true p in
    L.expect p ":=";
    Let (x, expression true p))
  else if L.eat_keyword p "if" then (
    let c = expression true p in
    L.expect_keyword p "then";
    let s = statement p in
    L.expect_keyword p "end";
    IfStmt (c, s))
  else if L.eat_keyword p "while" then (
    let c = expression true p in
    L.expect_keyword p "do";
    let s = statement p in
    L.expect_keyword p "end";
    While (c, s))
  else L.error p "statement expected"

and statement p =
  let rec loop s =
    if L.eat_symbol p ';' then
      let t = simple_stmt p in
      loop (Seq (s, t))
    else s
  in
  loop (simple_stmt p)

and block_contents p =
  let s = simple_stmt p in
  L.expect_symbol p ';';
  let saved = p.L.pos in
  match L.protect (fun () -> block_contents p) with
  | Ok (t, e) -> (Seq (s, t), e)
  | Error _ ->
      p.L.pos <- saved;
      (s, expression true p)

let parse_code = L.parse (expression false)
let parse_stmt_code = L.parse (expression true)

(* Rust's usize text codec accepts a leading '+'. This applies to tuple
   inputs and effects, rather than the expression lexer or JSON instructions. *)
let parse_natural_text text =
  let text = String.trim text in
  if String.length text > 0 && text.[0] = '+' then
    let digits = String.sub text 1 (String.length text - 1) in
    if
      digits <> ""
      && String.for_all (function '0' .. '9' -> true | _ -> false) digits
    then N.of_string digits
    else Error "Expected a natural number"
  else N.of_string text

let parse_ainput text =
  let text = String.trim text in
  let length = String.length text in
  if length < 2 || text.[0] <> '(' || text.[length - 1] <> ')' then
    Error "Expected to start with '(' and end with ')'"
  else
    let items = String.split_on_char ',' (String.sub text 1 (length - 2)) in
    let rec loop acc = function
      | [] -> Ok (List.rev acc)
      | x :: xs -> (
          if String.trim x = "" then loop acc xs
          else
            match parse_natural_text x with
            | Ok n -> loop (n :: acc) xs
            | Error e -> Error e)
    in
    loop [] items

let print_ainput ns = "(" ^ String.concat ", " (List.map N.to_string ns) ^ ")"

let parse_effect text =
  if String.trim text = "" then Ok None
  else Result.map Option.some (parse_natural_text text)

let print_effect = function None -> "" | Some n -> N.to_string n
let routput_json output = `String (print_effect output)
let foutput_json output = `String (N.to_string output)

let rec free_vars = function
  | Nat _ | Bool _ | Unit | Print _ -> Names.empty
  | Var x -> Names.singleton x
  | BinOp (a, _, b) | App (a, b) -> Names.union (free_vars a) (free_vars b)
  | UnOp (_, e) -> free_vars e
  | Fun (x, e) -> Names.remove x (free_vars e)
  | Rec (f, x, e) -> Names.remove f (Names.remove x (free_vars e))
  | If (c, t, e) ->
      Names.union (free_vars c) (Names.union (free_vars t) (free_vars e))
  | Block _ ->
      invalid_arg "substitution applies to the direct expression language"

let rec all_names = function
  | Fun (x, e) -> Names.add x (all_names e)
  | Rec (f, x, e) -> Names.add f (Names.add x (all_names e))
  | BinOp (a, _, b) | App (a, b) -> Names.union (all_names a) (all_names b)
  | UnOp (_, e) -> all_names e
  | If (c, t, e) ->
      Names.union (all_names c) (Names.union (all_names t) (all_names e))
  | e -> free_vars e

let fresh used replacement base =
  let rec loop i =
    let x = base ^ "_" ^ string_of_int i in
    if Names.mem x used || Names.mem x replacement then loop (i + 1) else x
  in
  loop 0

let rec rename e from into =
  match e with
  | Var x -> if x = from then Var into else e
  | Fun (x, _) when x = from -> e
  | Rec (f, x, _) when f = from || x = from -> e
  | Fun (x, b) -> Fun (x, rename b from into)
  | Rec (f, x, b) -> Rec (f, x, rename b from into)
  | BinOp (a, o, b) -> BinOp (rename a from into, o, rename b from into)
  | UnOp (o, b) -> UnOp (o, rename b from into)
  | App (a, b) -> App (rename a from into, rename b from into)
  | If (c, t, b) ->
      If (rename c from into, rename t from into, rename b from into)
  | _ -> e

let rec substitute e target replacement =
  match e with
  | Var x -> if x = target then replacement else e
  | Fun (x, _) when x = target -> e
  | Rec (f, x, _) when f = target || x = target -> e
  | Fun (x, b) ->
      let fv = free_vars replacement in
      let x, b =
        if Names.mem x fv then
          let y = fresh (Names.add target (all_names e)) fv x in
          (y, rename b x y)
        else (x, b)
      in
      Fun (x, substitute b target replacement)
  | Rec (f, x, b) ->
      let fv = free_vars replacement
      and used = Names.add target (all_names e) in
      let f, b =
        if Names.mem f fv then
          let y = fresh used fv f in
          (y, rename b f y)
        else (f, b)
      in
      let x, b =
        if Names.mem x fv then
          let y = fresh used fv x in
          (y, rename b x y)
        else (x, b)
      in
      Rec (f, x, substitute b target replacement)
  | BinOp (a, o, b) ->
      BinOp (substitute a target replacement, o, substitute b target replacement)
  | UnOp (o, b) -> UnOp (o, substitute b target replacement)
  | App (a, b) ->
      App (substitute a target replacement, substitute b target replacement)
  | If (c, t, b) ->
      If
        ( substitute c target replacement,
          substitute t target replacement,
          substitute b target replacement )
  | Block _ ->
      invalid_arg "substitution applies to the direct expression language"
  | _ -> e
