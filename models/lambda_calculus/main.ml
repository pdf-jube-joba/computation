open Utils_runtime.Machine
module Variable = Utils_names.Symbol.Make ()
module Var_id = Variable.Id
module Names = Variable.Names

type var = Variable.t = { id : Var_id.t; name : Utils_names.Identifier.t }
type term = Var of var | Abs of var * term | App of term * term
type code = term
type ainput = term list
type rinput = int
type routput = unit
type foutput = term
type state = term

let fresh_var = Variable.fresh

(* IDs are allocated locally while parsing. Combining independently parsed terms
   freshens their IDs before applying them. No process-global variable counter. *)
module Code = struct
  type t = term

  module L = Utils_parsing.Parser

  let read p =
    let reader = Variable.Reader.create () in
    let fresh name = Variable.Reader.bind reader name in
    let rec application env p =
      let first = atom env p in
      let rec rest left =
        let starts_atom =
          L.is_group p L.Parentheses
          ||
          match L.peek p with
          | L.Ident _ | L.Keyword "fun" -> true
          | _ -> false
        in
        if starts_atom then rest (App (left, atom env p)) else left
      in
      rest first
    and atom env p =
      match L.peek p with
      | L.Ident name ->
          ignore (L.bump p);
          let var =
            Variable.Reader.use reader env
              (Utils_names.Identifier.make_exn name)
          in
          Var var
      | _ when L.is_group p L.Parentheses ->
          L.group p L.Parentheses (application env)
      | L.Keyword "fun" ->
          ignore (L.bump p);
          let rec binders env acc =
            match L.peek p with
            | L.Ident name ->
                ignore (L.bump p);
                let var = fresh (Utils_names.Identifier.make_exn name) in
                binders
                  ((Utils_names.Identifier.make_exn name, var) :: env)
                  (var :: acc)
            | L.Symbol '.' when acc <> [] ->
                ignore (L.bump p);
                (env, List.rev acc)
            | _ -> L.error p "Expected binders followed by '.'"
          in
          let env, vars = binders env [] in
          let body = application env p in
          List.fold_right (fun var body -> Abs (var, body)) vars body
      | _ -> L.error p "Expected a lambda term"
    in
    application [] p

  let parse = L.parse read

  (* Allocate free names first, so alpha-renaming only changes binder hints
     when a binder and a free variable have the same preferred name. *)
  let print term =
    let names = ref Names.empty in
    let name var =
      let name, next = Names.assign ~hint:var.name var.id !names in
      names := next;
      name
    in
    let rec free bound = function
      | Var var -> if not (Var_id.Set.mem var.id bound) then ignore (name var)
      | Abs (var, body) -> free (Var_id.Set.add var.id bound) body
      | App (left, right) ->
          free bound left;
          free bound right
    in
    free Var_id.Set.empty term;
    let rec visit = function
      | Var var -> Utils_names.Identifier.to_string (name var)
      | Abs (var, body) ->
          let binder = name var in
          "\\fun " ^ Utils_names.Identifier.to_string binder ^ ". " ^ visit body
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
  let table = ref Var_id.Map.empty in
  let variable var =
    match Var_id.Map.find_opt var.id !table with
    | Some fresh -> fresh
    | None ->
        let fresh = { var with id = Var_id.Supply.take next } in
        table := Var_id.Map.add var.id fresh !table;
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
  let next = ref (Var_id.Supply.after (max_id code)) in
  Ok (List.fold_left (fun left arg -> App (left, freshen arg next)) code args)

(* Alpha-renaming changes bound variables only; free-variable identity survives
   reduction. Each binder occurrence gets a new ID, including shared subtrees. *)
let freshen_bound term next =
  let rec visit env = function
    | Var var ->
        Var
          (match List.assoc_opt var.id env with Some var -> var | None -> var)
    | Abs (var, body) ->
        let fresh = { var with id = Var_id.Supply.take next } in
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
    let next = ref (Var_id.Supply.after (max_id term)) in
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

module Snapshot = struct
  type t = state

  let to_yojson term =
    let var_json var =
      `Assoc
        [
          ("id", Var_id.to_yojson var.id);
          ("name", `String (Utils_names.Identifier.to_string var.name));
        ]
    in
    let rec visit = function
      | Var var -> `List [ `String "var"; var_json var ]
      | Abs (var, body) -> `List [ `String "abs"; var_json var; visit body ]
      | App (left, right) -> `List [ `String "app"; visit left; visit right ]
    in
    visit term

  let of_yojson json =
    let vars = ref Var_id.Map.empty and binders = ref Var_id.Set.empty in
    let variable = function
      | `Assoc fields -> (
          match (List.assoc_opt "id" fields, List.assoc_opt "name" fields) with
          | Some (`Int id), Some (`String name)
            when id >= 0 && id < max_int && Utils_names.Identifier.valid name
            -> (
              let* id = Var_id.of_int id in
              let* name = Utils_names.Identifier.make name in
              match Var_id.Map.find_opt id !vars with
              | Some var when var.name <> name ->
                  Error "Conflicting variable names"
              | Some var -> Ok var
              | None ->
                  let var = { id; name } in
                  vars := Var_id.Map.add id var !vars;
                  Ok var)
          | _ -> Error "Invalid variable in snapshot")
      | _ -> Error "Invalid variable in snapshot"
    in
    let rec visit env = function
      | `List [ `String "var"; json ] ->
          let* var = variable json in
          if Var_id.Set.mem var.id !binders && not (List.mem var.id env) then
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
          | Some (`Int id) -> (
              match Var_id.of_int id with
              | Ok id -> binders := Var_id.Set.add id !binders
              | Error _ -> ())
          | _ -> ());
          collect body
      | `List [ `String "app"; left; right ] ->
          collect left;
          collect right
      | _ -> ()
    in
    collect json;
    visit [] json
end

let render term =
  let rec redexes = function
    | Var _ -> []
    | Abs (_, body) -> redexes body
    | App (left, right) as term ->
        (match left with Abs _ -> [ term ] | _ -> [])
        @ redexes left @ redexes right
  in
  [
    Utils_runtime.Render.text ~title:"term" (Code.print term);
    Utils_runtime.Render.indexed_table ~title:"redexes"
      ~columns:[ "index"; "term" ] Code.print (redexes term);
  ]

module Ainput = struct
  type t = ainput

  let parse =
    Utils_parsing.Parser.parse (fun p ->
        Utils_parsing.Parser.separated_list p ~separator:',' Code.read)

  let print = fun terms -> String.concat ", " (List.map Code.print terms)
end

module Rinput = Utils_data.Text.Natural_int
module Routput = Utils_data.Text.Unit

module Foutput = struct
  type t = foutput

  let parse = Code.parse
  let print = Code.print
end
