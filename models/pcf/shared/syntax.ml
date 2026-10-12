module N = Utils.Number
module L = Utils.Parser
module Names = Utils.Identifier.Set

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

module Source_code = struct
  type t = expr

  let print_binop = function Add -> "+" | Sub -> "-" | And -> "&&"
  let print_unop = function Inc -> "inc" | Dec -> "dec" | Not -> "not"

  let rec print = function
    | Nat n -> N.to_string n
    | Bool true -> "#true"
    | Bool false -> "#false"
    | Unit -> "#unit"
    | Print n -> "print " ^ N.to_string n
    | Var x -> x
    | BinOp (a, o, b) ->
        "(" ^ print a ^ " " ^ print_binop o ^ " " ^ print b ^ ")"
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

  let reserved statements x =
    List.mem x
      ([
         "print"; "fun"; "rec"; "if"; "then"; "else"; "fi"; "inc"; "dec"; "not";
       ]
      @ if statements then [ "skip"; "let"; "while"; "do"; "end" ] else [])

  let identifier statements p =
    let x = L.identifier p in
    if reserved statements x then
      L.error p ("identifier expected, found keyword '" ^ x ^ "'");
    x

  let number = L.natural

  let rec expression statements p =
    if L.peek p = L.Ident "if" then if_expr statements p
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
      if L.peek p = L.Symbol '(' then
        let x = L.group p ~opening:'(' (expression statements) in
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
    | L.Symbol '#' -> (
        ignore (L.bump p);
        match L.identifier p with
        | "true" -> Bool true
        | "false" -> Bool false
        | "unit" -> Unit
        | k -> L.error p ("unknown token: #" ^ k))
    | L.Ident "print" when not statements ->
        ignore (L.bump p);
        Print (number p)
    | L.Ident "fun" ->
        ignore (L.bump p);
        let x = identifier statements p in
        L.expect p "=>";
        Fun (x, expression statements p)
    | L.Ident "rec" ->
        ignore (L.bump p);
        let f = identifier statements p in
        let x = identifier statements p in
        L.expect p "=>";
        Rec (f, x, expression statements p)
    | L.Ident "if" -> if_expr statements p
    | L.Symbol '(' -> L.group p ~opening:'(' (expression statements)
    | L.Symbol '{' when statements ->
        let s, e = L.group p ~opening:'{' block_contents in
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
end

let is_value = function
  | Nat _ | Bool _ | Unit | Fun _ | Rec _ -> true
  | _ -> false

let expect_nat = function
  | Nat n -> Ok n
  | _ -> Error "final value is not a natural number"

let apply_inputs e inputs = List.fold_left (fun e n -> App (e, Nat n)) e inputs

module Stmt_code = struct
  include Source_code

  let parse = L.parse (expression true)
end

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

module Substitution =
  Utils.Binding.Make
    (struct
      type t = expr

      let variable name = Var name

      let view e =
        let open Utils.Binding in
        match e with
        | Var name -> Variable name
        | Fun (x, body) ->
            node ~binders:[ x ]
              [ ([ 0 ], body) ]
              (fun names children -> Fun (List.hd names, List.hd children))
        | Rec (f, x, body) ->
            node ~binders:[ f; x ]
              [ ([ 0; 1 ], body) ]
              (fun names children ->
                Rec (List.nth names 0, List.nth names 1, List.hd children))
        | BinOp (a, op, b) ->
            node
              [ ([], a); ([], b) ]
              (fun _ children ->
                BinOp (List.nth children 0, op, List.nth children 1))
        | UnOp (op, body) ->
            node [ ([], body) ] (fun _ children -> UnOp (op, List.hd children))
        | App (a, b) ->
            node
              [ ([], a); ([], b) ]
              (fun _ children -> App (List.nth children 0, List.nth children 1))
        | If (c, a, b) ->
            node
              [ ([], c); ([], a); ([], b) ]
              (fun _ children ->
                If
                  (List.nth children 0, List.nth children 1, List.nth children 2))
        | Block _ ->
            invalid_arg "substitution applies to the direct expression language"
        | _ -> node [] (fun _ _ -> e)
    end)
    ()

let substitute = Substitution.substitute

module Code = struct
  include Source_code

  let parse text = Result.map Substitution.normalize (Source_code.parse text)
end

module Ainput = Utils.Text.Natural_list

module Routput = struct
  type t = print_effect

  let parse text =
    if String.trim text = "" then Ok None
    else Result.map Option.some (N.of_string_with_plus text)

  let print = function None -> "" | Some n -> N.to_string n
end

module Foutput = Utils.Text.Natural
