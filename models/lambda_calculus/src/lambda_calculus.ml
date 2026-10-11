open Utils.Machine

type var = { id : int; name : Utils.Identifier.t }
type term = Var of var | Abs of var * term | App of term * term
type code = term
type ainput = term list
type rinput = int
type routput = unit
type foutput = term
type state = term

(* IDs are allocated locally while parsing. Combining independently parsed terms
   freshens their IDs before applying them. No process-global variable counter. *)
module Code = struct
  type t = term
  type token = Name of string | Lambda | Dot | Left | Right | End

  exception Parse_error of string

  let tokenize text =
    let length = String.length text in
    let has = Utils.String_ext.starts_at text in
    let start = Utils.Identifier.is_start in
    let continue = Utils.Identifier.is_continue in
    let rec scan i acc =
      if i = length then List.rev (End :: acc)
      else
        match text.[i] with
        | c when Utils.String_ext.is_whitespace c -> scan (i + 1) acc
        | '/' when has i "//" ->
            scan (Utils.String_ext.skip_line text (i + 2)) acc
        | '/' when has i "/*" ->
            scan (Utils.Lexer.skip_block_comment text i) acc
        | '\\' -> scan (i + 1) (Lambda :: acc)
        | '.' -> scan (i + 1) (Dot :: acc)
        | '(' -> scan (i + 1) (Left :: acc)
        | ')' -> scan (i + 1) (Right :: acc)
        | c when start c ->
            let j = Utils.String_ext.skip_while continue text (i + 1) in
            scan j (Name (String.sub text i (j - i)) :: acc)
        | _ ->
            raise
              (Parse_error (Printf.sprintf "Unexpected character at byte %d" i))
    in
    Array.of_list (scan 0 [])

  let parse text =
    try
      let tokens = tokenize text in
      let pos = ref 0 and next_id = ref 0 in
      let free = Hashtbl.create 8 in
      let fresh name =
        let id = !next_id in
        incr next_id;
        { id; name }
      in
      let peek () = tokens.(!pos) in
      let take () =
        let token = peek () in
        incr pos;
        token
      in
      let fail message =
        raise (Parse_error (Printf.sprintf "%s at token %d" message !pos))
      in
      let rec application env =
        let first = atom env in
        let rec rest left =
          match peek () with
          | Name _ | Left | Lambda -> rest (App (left, atom env))
          | _ -> left
        in
        rest first
      and atom env =
        match take () with
        | Name name ->
            let var =
              match List.assoc_opt name env with
              | Some var -> var
              | None -> (
                  match Hashtbl.find_opt free name with
                  | Some var -> var
                  | None ->
                      let var = fresh name in
                      Hashtbl.add free name var;
                      var)
            in
            Var var
        | Left ->
            let term = application env in
            if take () <> Right then fail "Expected ')'";
            term
        | Lambda ->
            let rec binders env acc =
              match peek () with
              | Name name ->
                  ignore (take ());
                  let var = fresh name in
                  binders ((name, var) :: env) (var :: acc)
              | Dot when acc <> [] ->
                  ignore (take ());
                  (env, List.rev acc)
              | _ -> fail "Expected binders followed by '.'"
            in
            let env, vars = binders env [] in
            let body = application env in
            List.fold_right (fun var body -> Abs (var, body)) vars body
        | _ -> fail "Expected a lambda term"
      in
      let term = application [] in
      if peek () <> End then fail "Unexpected trailing tokens";
      Ok term
    with Parse_error message | Utils.Lexer.Error message -> Error message

  (* Use distinct printable names for distinct IDs, so printing a shared term
   never introduces apparent variable capture. *)
  let print term =
    let names = Hashtbl.create 16 and used = Hashtbl.create 16 in
    let name var =
      match Hashtbl.find_opt names var.id with
      | Some name -> name
      | None ->
          let rec choose name =
            if Hashtbl.mem used name then choose (name ^ "_") else name
          in
          let name = choose var.name in
          Hashtbl.add names var.id name;
          Hashtbl.add used name ();
          name
    in
    let rec visit = function
      | Var var -> name var
      | Abs (var, body) ->
          let binder = name var in
          "\\" ^ binder ^ ". " ^ visit body
      | App (left, right) ->
          let operand term =
            match term with Abs _ -> "(" ^ visit term ^ ")" | _ -> visit term
          in
          let left = operand left in
          "(" ^ left ^ " " ^ operand right ^ ")"
    in
    visit term
end

let rec max_id = function
  | Var var -> var.id
  | Abs (var, body) -> max var.id (max_id body)
  | App (left, right) -> max (max_id left) (max_id right)

let freshen term next =
  let table = Hashtbl.create 16 in
  let variable var =
    match Hashtbl.find_opt table var.id with
    | Some fresh -> fresh
    | None ->
        let fresh = { var with id = !next } in
        incr next;
        Hashtbl.add table var.id fresh;
        fresh
  in
  let rec visit = function
    | Var var -> Var (variable var)
    | Abs (var, body) ->
        let var = variable var in
        Abs (var, visit body)
    | App (left, right) ->
        let left = visit left in
        App (left, visit right)
  in
  visit term

let make code args =
  let next = ref (max_id code + 1) in
  Ok (List.fold_left (fun left arg -> App (left, freshen arg next)) code args)

(* Alpha-renaming changes bound variables only; free-variable identity survives
   reduction. Each binder occurrence gets a new ID, including shared subtrees. *)
let freshen_bound term next =
  let rec visit env = function
    | Var var ->
        Var
          (match List.assoc_opt var.id env with Some var -> var | None -> var)
    | Abs (var, body) ->
        let fresh = { var with id = !next } in
        incr next;
        Abs (fresh, visit ((var.id, fresh) :: env) body)
    | App (left, right) ->
        let left = visit env left in
        App (left, visit env right)
  in
  visit [] term

let rec substitute term variable replacement =
  match term with
  | Var var -> if var.id = variable.id then replacement else term
  | Abs (var, _) when var.id = variable.id -> term
  | Abs (var, body) -> Abs (var, substitute body variable replacement)
  | App (left, right) ->
      App
        ( substitute left variable replacement,
          substitute right variable replacement )

let rec redex_count = function
  | Var _ -> 0
  | Abs (_, body) -> redex_count body
  | App (left, right) ->
      (match left with Abs _ -> 1 | _ -> 0)
      + redex_count left + redex_count right

(* Preorder, left-to-right, zero-based, as in the Rust model. Freshening the
   abstraction before substitution prevents capture even when subtrees share IDs. *)
let reduce term index =
  if index < 0 then Error "Redex index must be nonnegative"
  else
    let remaining = ref index and reduced = ref false in
    let next = ref (max_id term + 1) in
    let rec visit term =
      if !reduced then term
      else
        match term with
        | Var _ -> term
        | Abs (var, body) -> Abs (var, visit body)
        | App (left, right) -> (
            match left with
            | Abs _ when !remaining = 0 -> (
                reduced := true;
                match freshen_bound left next with
                | Abs (var, body) -> substitute body var right
                | _ -> assert false)
            | _ ->
                (match left with Abs _ -> decr remaining | _ -> ());
                let left = visit left in
                App (left, visit right))
    in
    let result = visit term in
    if !reduced then Ok result else Error "No redex found at the given index"

let step term index =
  let* next = reduce term index in
  if redex_count next = 0 then Ok (Halt next)
  else Ok (Continue { next; output = () })

let snapshot term =
  let var_json var =
    `Assoc [ ("id", `Int var.id); ("name", `String var.name) ]
  in
  let rec visit = function
    | Var var -> `List [ `String "var"; var_json var ]
    | Abs (var, body) -> `List [ `String "abs"; var_json var; visit body ]
    | App (left, right) -> `List [ `String "app"; visit left; visit right ]
  in
  visit term

let restore json =
  let vars = Hashtbl.create 16 and binders = Hashtbl.create 16 in
  let variable = function
    | `Assoc fields -> (
        match (List.assoc_opt "id" fields, List.assoc_opt "name" fields) with
        | Some (`Int id), Some (`String name)
          when id >= 0 && id < max_int && Utils.Identifier.valid name -> (
            match Hashtbl.find_opt vars id with
            | Some var when var.name <> name ->
                Error "Conflicting variable names"
            | Some var -> Ok var
            | None ->
                let var = { id; name } in
                Hashtbl.add vars id var;
                Ok var)
        | _ -> Error "Invalid variable in snapshot")
    | _ -> Error "Invalid variable in snapshot"
  in
  let rec visit env = function
    | `List [ `String "var"; json ] ->
        let* var = variable json in
        if Hashtbl.mem binders var.id && not (List.mem var.id env) then
          Error "Bound variable occurs outside its scope"
        else Ok (Var var)
    | `List [ `String "abs"; json; body ] ->
        let* var = variable json in
        if List.mem var.id env then Error "Repeated binder ID"
        else
          let* body = visit (var.id :: env) body in
          Ok (Abs (var, body))
    | `List [ `String "app"; left; right ] ->
        let* left = visit env left in
        let* right = visit env right in
        Ok (App (left, right))
    | _ -> Error "Invalid lambda snapshot"
  in
  let rec collect = function
    | `List [ `String "abs"; `Assoc fields; body ] ->
        (match List.assoc_opt "id" fields with
        | Some (`Int id) -> Hashtbl.replace binders id ()
        | _ -> ());
        collect body
    | `List [ `String "app"; left; right ] ->
        collect left;
        collect right
    | _ -> ()
  in
  collect json;
  visit [] json

let render term =
  let rec redexes = function
    | Var _ -> []
    | Abs (_, body) -> redexes body
    | App (left, right) as term ->
        (match left with Abs _ -> [ term ] | _ -> [])
        @ redexes left @ redexes right
  in
  Utils.Render.state
    [
      Utils.Render.text ~title:"term" (Code.print term);
      Utils.Render.indexed_table ~title:"redexes" ~columns:[ "index"; "term" ]
        Code.print (redexes term);
    ]

let assoc_app = List.fold_left (fun left right -> App (left, right))

let lambdas vars body =
  List.fold_right (fun var body -> Abs (var, body)) vars body

let free_variables term =
  let rec visit bound acc = function
    | Var var ->
        if List.mem var.id bound || List.exists (fun v -> v.id = var.id) acc
        then acc
        else var :: acc
    | Abs (var, body) -> visit (var.id :: bound) acc body
    | App (left, right) -> visit bound (visit bound acc left) right
  in
  List.rev (visit [] [] term)

let bounded_variables term =
  let rec visit acc = function
    | Var _ -> acc
    | Abs (var, body) ->
        visit
          (if List.exists (fun v -> v.id = var.id) acc then acc else var :: acc)
          body
    | App (left, right) -> visit (visit acc left) right
  in
  List.rev (visit [] term)

let alpha_eq left right =
  let rec position id i = function
    | [] -> None
    | h :: t -> if h = id then Some i else position id (i + 1) t
  in
  let rec equal env1 env2 left right =
    match (left, right) with
    | Var v, Var w -> (
        match (position v.id 0 env1, position w.id 0 env2) with
        | Some i, Some j -> i = j
        | None, None -> v.id = w.id
        | _ -> false)
    | Abs (v, left), Abs (w, right) ->
        equal (v.id :: env1) (w.id :: env2) left right
    | App (l, r), App (l', r') -> equal env1 env2 l l' && equal env1 env2 r r'
    | _ -> false
  in
  equal [] [] left right

let alpha_conversion term = freshen_bound term (ref (max_id term + 1))
let unchecked_subst = substitute
let var_change before after term = substitute term before (Var after)
let is_normal_form term = redex_count term = 0

let rec normalize ?(fuel = 100000) term =
  if is_normal_form term then Ok term
  else if fuel = 0 then Error "Normalization step limit reached"
  else
    let* next = reduce term 0 in
    normalize ~fuel:(fuel - 1) next

module Ainput = struct
  type t = ainput

  let parse text =
    if String.trim text = "" then Ok []
    else
      let rec loop = function
        | [] -> Ok []
        | text :: rest ->
            let* term = Code.parse text in
            let* terms = loop rest in
            Ok (term :: terms)
      in
      loop (String.split_on_char ',' text)

  let print = fun terms -> String.concat ", " (List.map Code.print terms)
end

module Rinput = Utils.Text.Natural_int
module Routput = Utils.Text.Unit

module Foutput = struct
  type t = foutput

  let parse = Code.parse
  let print = Code.print
end
