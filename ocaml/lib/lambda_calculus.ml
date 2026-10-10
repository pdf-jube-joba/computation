open Machine

type var = { id : int; name : string }
type term = Var of var | Abs of var * term | App of term * term
type code = term
type ainput = term list
type rinput = int
type routput = unit
type foutput = term
type state = term

(* IDs are allocated locally while parsing. Combining independently parsed terms
   freshens their IDs before applying them. No process-global variable counter. *)
type token = Name of string | Lambda | Dot | Left | Right | End

exception Parse_error of string

let tokenize text =
  let length = String.length text in
  let start = function 'a' .. 'z' | 'A' .. 'Z' | '_' -> true | _ -> false in
  let continue c =
    start c || match c with '0' .. '9' | '-' | '\'' -> true | _ -> false
  in
  let rec scan i acc =
    if i = length then List.rev (End :: acc)
    else
      match text.[i] with
      | ' ' | '\t' | '\r' | '\n' -> scan (i + 1) acc
      | '\\' -> scan (i + 1) (Lambda :: acc)
      | '.' -> scan (i + 1) (Dot :: acc)
      | '(' -> scan (i + 1) (Left :: acc)
      | ')' -> scan (i + 1) (Right :: acc)
      | c when start c ->
          let j = ref (i + 1) in
          while !j < length && continue text.[!j] do
            incr j
          done;
          scan !j (Name (String.sub text i (!j - i)) :: acc)
      | _ ->
          raise
            (Parse_error (Printf.sprintf "Unexpected character at byte %d" i))
  in
  Array.of_list (scan 0 [])

let parse_code text =
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
  with Parse_error message -> Error message

let parse_ainput text =
  if String.trim text = "" then Ok []
  else
    let rec loop = function
      | [] -> Ok []
      | text :: rest ->
          let* term = parse_code text in
          let* terms = loop rest in
          Ok (term :: terms)
    in
    loop (String.split_on_char ',' text)

let parse_rinput = natural

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

(* Use distinct printable names for distinct IDs, so printing a shared term
   never introduces apparent variable capture. *)
let print term =
  let names = Hashtbl.create 16 and used = Hashtbl.create 16 in
  let name var =
    match Hashtbl.find_opt names var.id with
    | Some name -> name
    | None ->
        let rec choose name =
          if Hashtbl.mem used name then choose (name ^ "'") else name
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
          when id >= 0 && id < max_int && name <> "" -> (
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

let render term = text_render (print term)
let routput_json () = `Null
let foutput_json term = `String (print term)
