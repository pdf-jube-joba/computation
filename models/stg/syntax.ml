module Id = Utils_names.Identifier
module N = Utils_base.Number
module P = Utils_parsing.Parser

type atom = Var of Id.t | Lit of N.t [@@deriving yojson]
type update = Updatable | Reentrant [@@deriving yojson]
type prim = Add | Sub | Mul | Eq | Lt [@@deriving yojson]

type expr =
  | App of Id.t * atom list
  | Con of Id.t * atom list
  | Nat of N.t
  | Prim of prim * atom * atom
  | Let of bool * binding list * expr
  | Case of expr * alternative list

and lambda = {
  free : Id.t list;
  update : update;
  args : Id.t list;
  body : expr;
}

and binding = Id.t * lambda

and pattern =
  | Constructor of Id.t * Id.t list
  | Literal of N.t
  | Default of Id.t option

and alternative = pattern * expr [@@deriving yojson]

type t = binding list [@@deriving yojson]

let list p delimiter read =
  P.group p delimiter (fun p -> P.separated_list p ~separator:',' read)

let read_atom p =
  match P.peek p with
  | P.Ident _ -> Var (P.identifier p)
  | P.Number _ -> Lit (P.natural p)
  | _ -> P.error p "Expected a variable or natural number"

let rec read_expr p =
  if P.is_group p P.Parentheses then P.group p P.Parentheses read_expr
  else
    match P.peek p with
    | P.Number _ -> Nat (P.natural p)
    | P.Ident _ ->
        let name = P.identifier p in
        App (name, list p P.Parentheses read_atom)
    | P.Keyword "con" ->
        P.expect_keyword p "con";
        let name = P.identifier p in
        Con (name, list p P.Parentheses read_atom)
    | P.Keyword ("let" | "letrec") ->
        let recursive = P.keyword p = "letrec" in
        let bindings = P.group p P.Braces read_bindings in
        P.expect_keyword p "in";
        Let (recursive, bindings, read_expr p)
    | P.Keyword "case" ->
        P.expect_keyword p "case";
        let scrutinee = read_expr p in
        P.expect_keyword p "of";
        let choices =
          P.group p P.Braces (fun p ->
              P.separated_list ~allow_trailing:true p ~separator:';'
                read_alternative)
        in
        Case (scrutinee, choices)
    | P.Keyword ("add" | "sub" | "mul" | "eq" | "lt") -> (
        let op =
          match P.keyword p with
          | "add" -> Add
          | "sub" -> Sub
          | "mul" -> Mul
          | "eq" -> Eq
          | _ -> Lt
        in
        match list p P.Parentheses read_atom with
        | [ a; b ] -> Prim (op, a, b)
        | _ -> P.error p "Primitive operations require two atoms")
    | _ -> P.error p "Expected an STG expression"

and read_binding p =
  let name = P.identifier p in
  P.expect_symbol p '=';
  P.expect_keyword p "fun";
  let free = list p P.Brackets P.identifier in
  let update =
    match P.keyword p with
    | "u" -> Updatable
    | "n" -> Reentrant
    | _ -> P.error p "Expected update flag \\u or \\n"
  in
  let args = list p P.Brackets P.identifier in
  P.expect p "->";
  (name, { free; update; args; body = read_expr p })

and read_bindings p =
  P.separated_list ~allow_trailing:true p ~separator:';' read_binding

and read_alternative p =
  let pattern =
    match P.peek p with
    | P.Number _ -> Literal (P.natural p)
    | P.Ident _ ->
        let name = P.identifier p in
        Constructor (name, list p P.Parentheses P.identifier)
    | P.Keyword "default" ->
        P.expect_keyword p "default";
        let binder =
          match P.peek p with P.Ident _ -> Some (P.identifier p) | _ -> None
        in
        Default binder
    | _ -> P.error p "Expected a constructor, literal or \\default alternative"
  in
  P.expect p "->";
  (pattern, read_expr p)

let names xs = String.concat ", " (List.map Id.to_string xs)
let atom_text = function Var x -> Id.to_string x | Lit n -> N.to_string n
let atoms xs = "(" ^ String.concat ", " (List.map atom_text xs) ^ ")"

let prim_text = function
  | Add -> "add"
  | Sub -> "sub"
  | Mul -> "mul"
  | Eq -> "eq"
  | Lt -> "lt"

let rec expr_text = function
  | App (f, xs) -> Id.to_string f ^ atoms xs
  | Con (c, xs) -> "\\con " ^ Id.to_string c ^ atoms xs
  | Nat n -> N.to_string n
  | Prim (op, a, b) -> "\\" ^ prim_text op ^ atoms [ a; b ]
  | Let (recursive, bs, body) ->
      (if recursive then "\\letrec" else "\\let")
      ^ " { " ^ print bs ^ " } \\in " ^ expr_text body
  | Case (e, alts) ->
      "\\case " ^ expr_text e ^ " \\of { "
      ^ String.concat "; "
          (List.map (fun (p, e) -> pattern_text p ^ " -> " ^ expr_text e) alts)
      ^ " }"

and pattern_text = function
  | Constructor (c, xs) -> Id.to_string c ^ "(" ^ names xs ^ ")"
  | Literal n -> N.to_string n
  | Default None -> "\\default"
  | Default (Some x) -> "\\default " ^ Id.to_string x

and lambda_text lf =
  "\\fun [" ^ names lf.free ^ "] "
  ^ (match lf.update with Updatable -> "\\u" | Reentrant -> "\\n")
  ^ " [" ^ names lf.args ^ "] -> " ^ expr_text lf.body

and print bs =
  String.concat "; "
    (List.map (fun (x, lf) -> Id.to_string x ^ " = " ^ lambda_text lf) bs)

let unique label xs =
  Id.check_unique
    ~message:(fun x -> "Duplicate " ^ label ^ ": " ^ Id.to_string x)
    xs

let bound scope x =
  if not (List.mem x scope) then failwith ("Unbound variable: " ^ Id.to_string x)

(* A lambda's body can see only its explicit captures, parameters and globals.
   The allocation scope is used solely to check its capture list. *)
let rec check_expr globals scope = function
  | Nat _ -> ()
  | App (f, xs) ->
      bound scope f;
      List.iter (check_atom scope) xs
  | Con (_, xs) -> List.iter (check_atom scope) xs
  | Prim (_, a, b) ->
      check_atom scope a;
      check_atom scope b
  | Let (recursive, bs, body) ->
      if bs = [] then failwith "Empty let binding group";
      let names = List.map fst bs in
      unique "binding" names;
      let inner = names @ scope in
      List.iter
        (fun (_, lf) ->
          check_lambda globals (if recursive then inner else scope) lf)
        bs;
      check_expr globals inner body
  | Case (e, alts) ->
      check_expr globals scope e;
      check_alternatives globals scope alts

and check_atom scope = function Var x -> bound scope x | Lit _ -> ()

and check_lambda globals scope lf =
  unique "capture/parameter" (lf.free @ lf.args);
  if lf.update = Updatable && lf.args <> [] then
    failwith "An updatable lambda must have no parameters";
  List.iter (bound scope) lf.free;
  (* A local that shadows a global must still be captured explicitly. *)
  let globals =
    List.filter
      (fun x -> List.length (List.filter (Id.equal x) scope) <= 1)
      globals
  in
  check_expr globals (lf.args @ lf.free @ globals) lf.body

and check_alternatives globals scope alts =
  if alts = [] then failwith "Empty case alternatives";
  let keys =
    List.map
      (fun (p, _) ->
        match p with
        | Constructor (c, _) -> "con:" ^ Id.to_string c
        | Literal n -> "nat:" ^ N.to_string n
        | Default _ -> "default")
      alts
  in
  if List.length (List.sort_uniq String.compare keys) <> List.length keys then
    failwith "Duplicate case alternative";
  List.iter
    (fun (p, e) ->
      let vars =
        match p with
        | Constructor (_, xs) -> xs
        | Default (Some x) -> [ x ]
        | _ -> []
      in
      unique "pattern variable" vars;
      check_expr globals (vars @ scope) e)
    alts

let validate code =
  Utils_base.Error.protect (fun () ->
      let globals = List.map fst code in
      unique "global binding" globals;
      if not (List.mem (Id.make_exn "main") globals) then
        failwith "Missing main binding";
      List.iter
        (fun (_, lf) ->
          if lf.free <> [] then
            failwith "Top-level lambdas cannot capture local variables";
          check_lambda globals globals lf)
        code;
      let main = List.assoc (Id.make_exn "main") code in
      if main.args <> [] then failwith "main must have no parameters")

let parse source =
  let open Utils_base.Error in
  let* code = P.parse read_bindings source in
  let* () = validate code in
  Ok code
