module L = Utils.Lexer
module N = Utils.Number

let value = function Ok v -> v | Error e -> failwith e

type aatom = Var of string | Imm of N.t [@@deriving yojson]
type abinop = Add | Sub | Mul [@@deriving yojson]
type aexp = Atom of aatom | Bin of aexp * abinop * aexp [@@deriving yojson]
type relop = Eq | Ne | Lt [@@deriving yojson]

type bexp =
  | Not of bexp
  | Rel of aexp * relop * aexp
  | And of bexp * bexp
  | Or of bexp * bexp
  | Done of string
[@@deriving yojson]

type stmt =
  | Assign of string * aexp
  | IfGoto of bexp * int
  | Goto of int
  | Call of string
  | Run of string * string
  | Yield
[@@deriving yojson]

type fn_decl = { name : string; body : stmt list } [@@deriving yojson]
type code = fn_decl list [@@deriving yojson]

type global_env = {
  vars : (string * N.t) list;
  task_ids : (string * int option) list;
}
[@@deriving yojson]

type frame = { function_name : string; pc : int } [@@deriving yojson]
type ainput = int
type rinput = int
type routput = string
type foutput = global_env

type state = {
  code : code;
  env : global_env;
  workers : int option array;
  queue : int list;
  tasks : frame list array;
}
[@@deriving yojson]

let rec aexp_to_text = function
  | Atom (Var n) -> n
  | Atom (Imm n) -> N.to_string n
  | Bin (a, op, b) ->
      "(" ^ aexp_to_text a ^ " "
      ^ (match op with Add -> "+" | Sub -> "-" | Mul -> "*")
      ^ " " ^ aexp_to_text b ^ ")"

let rec bexp_to_text = function
  | Not b -> "!" ^ bexp_to_text b
  | Rel (a, op, b) ->
      aexp_to_text a ^ " "
      ^ (match op with Eq -> "==" | Ne -> "!=" | Lt -> "<")
      ^ " " ^ aexp_to_text b
  | And (a, b) -> "(" ^ bexp_to_text a ^ " && " ^ bexp_to_text b ^ ")"
  | Or (a, b) -> "(" ^ bexp_to_text a ^ " || " ^ bexp_to_text b ^ ")"
  | Done n -> "done $" ^ n

let fmt_offset n = if n >= 0 then "+" ^ string_of_int n else string_of_int n

let stmt_to_text = function
  | Assign (n, a) -> n ^ " <- " ^ aexp_to_text a
  | IfGoto (b, n) -> "if " ^ bexp_to_text b ^ " goto " ^ fmt_offset n
  | Goto n -> "goto " ^ fmt_offset n
  | Call n -> "call " ^ n
  | Run (n, id) -> "run " ^ n ^ " -> $" ^ id
  | Yield -> "yield"

let print code =
  String.concat "\n"
    (List.map
       (fun f ->
         "fn " ^ f.name ^ " {\n"
         ^ (if f.body = [] then ""
            else "  " ^ String.concat "; " (List.map stmt_to_text f.body) ^ "\n")
         ^ "}\n")
       code)

let rec parse_aexp_bp p min_bp =
  let lhs =
    match L.next p with
    | L.Ident n -> Atom (Var n)
    | L.Number n -> Atom (Imm (value (N.of_string n)))
    | L.Symbol '(' ->
        let a = parse_aexp_bp p 0 in
        L.expect_symbol p ')';
        a
    | _ -> L.error p "Unexpected token in arithmetic expression"
  in
  let rec loop lhs =
    let op =
      match L.peek p with
      | L.Symbol '+' -> Some (Add, 1, 2)
      | L.Symbol '-' -> Some (Sub, 1, 2)
      | L.Symbol '*' -> Some (Mul, 3, 4)
      | _ -> None
    in
    match op with
    | Some (op, lbp, rbp) when lbp >= min_bp ->
        ignore (L.next p);
        let rhs = parse_aexp_bp p rbp in
        loop (Bin (lhs, op, rhs))
    | _ -> lhs
  in
  loop lhs

let parse_aexp p = parse_aexp_bp p 0

let rec parse_bexp_bp p min_bp =
  let relation lhs =
    let op =
      if L.eat p "==" then Eq
      else if L.eat p "!=" then Ne
      else if L.eat_symbol p '<' then Lt
      else L.error p "Expected relational operator"
    in
    let rhs = parse_aexp p in
    Rel (lhs, op, rhs)
  in
  let lhs =
    if L.eat_symbol p '!' then Not (parse_bexp_bp p 5)
    else if L.eat_keyword p "done" then (
      L.expect_symbol p '$';
      Done (L.identifier p))
    else if L.eat_symbol p '(' then (
      (* Preserve the original boolean grouping, and accept printed arithmetic LHS. *)
      let saved = p.L.pos in
      try
        let b = parse_bexp_bp p 0 in
        L.expect_symbol p ')';
        b
      with L.Error _ ->
        p.L.pos <- saved;
        let a = parse_aexp p in
        L.expect_symbol p ')';
        relation a)
    else
      match L.next p with
      | L.Ident n -> relation (Atom (Var n))
      | L.Number n -> relation (Atom (Imm (value (N.of_string n))))
      | _ -> L.error p "Unexpected token in boolean expression"
  in
  let rec loop lhs =
    if min_bp <= 2 && L.eat p "&&" then
      let rhs = parse_bexp_bp p 3 in
      loop (And (lhs, rhs))
    else if min_bp <= 0 && L.eat p "||" then
      let rhs = parse_bexp_bp p 1 in
      loop (Or (lhs, rhs))
    else lhs
  in
  loop lhs

(* Function, assignment, call and run names are validated as whole strings
   by the Rust statement parser; those names may contain hyphens. Expression
   identifiers retain the original lexer rules. *)
let statement_identifier p =
  let name = L.identifier p in
  let rec suffix name =
    if L.peek p = L.Symbol '-' && fst p.L.tokens.(p.L.pos + 1) <> L.Symbol '>'
    then (
      ignore (L.next p);
      let name = name ^ "-" in
      let name =
        match L.peek p with
        | L.Ident n | L.Number n ->
            ignore (L.next p);
            name ^ n
        | _ -> name
      in
      suffix name)
    else value (Utils.Identifier.make name)
  in
  suffix name

let parse_offset p =
  let sign =
    if L.eat_symbol p '-' then -1
    else (
      ignore (L.eat_symbol p '+');
      1)
  in
  sign * L.integer p

let parse_stmt p =
  if L.eat_keyword p "if" then (
    let b = parse_bexp_bp p 0 in
    L.expect_keyword p "goto";
    IfGoto (b, parse_offset p))
  else if L.eat_keyword p "goto" then Goto (parse_offset p)
  else if L.eat_keyword p "call" then Call (statement_identifier p)
  else if L.eat_keyword p "run" then (
    let name = statement_identifier p in
    L.expect p "->";
    L.expect_symbol p '$';
    let id = statement_identifier p in
    Run (name, id))
  else if L.eat_keyword p "yield" then Yield
  else
    let n = statement_identifier p in
    L.expect p "<-";
    let a = parse_aexp p in
    Assign (n, a)

let parse_code =
  L.parse (fun p ->
      let rec functions acc =
        if L.eof p then List.rev acc
        else (
          L.expect_keyword p "fn";
          let name = statement_identifier p in
          L.expect_symbol p '{';
          let rec body acc =
            if L.eat_symbol p '}' then List.rev acc
            else if L.eat_symbol p ';' then body acc
            else
              let s = parse_stmt p in
              if L.peek p <> L.Symbol '}' then L.expect_symbol p ';';
              body (s :: acc)
          in
          let body = body [] in
          functions ({ name; body } :: acc))
      in
      let code = functions [] in
      if code = [] then L.error p "At least one function is required";
      code)

let parse_ainput = Utils.Machine.natural
let parse_rinput = Utils.Machine.natural

let set entries name v =
  List.sort
    (fun (a, _) (b, _) -> String.compare a b)
    ((name, v) :: List.remove_assoc name entries)

let parse_env text =
  L.protect (fun () ->
      List.fold_left
        (fun env line ->
          let line = String.trim line in
          if line = "" then env
          else
            match String.index_opt line '=' with
            | None -> failwith ("invalid env line: " ^ line)
            | Some i ->
                let name = String.trim (String.sub line 0 i)
                and v =
                  String.trim
                    (String.sub line (i + 1) (String.length line - i - 1))
                in
                if String.starts_with ~prefix:"$" name then (
                  let name = String.sub name 1 (String.length name - 1) in
                  ignore (value (Utils.Identifier.make name));
                  let id =
                    if String.lowercase_ascii v = "none" then None
                    else Some (value (Utils.Machine.natural v))
                  in
                  { env with task_ids = set env.task_ids name id })
                else (
                  ignore (value (Utils.Identifier.make name));
                  {
                    env with
                    vars =
                      set env.vars name
                        (value
                           (N.of_string
                              (if String.starts_with ~prefix:"+" v then
                                 String.sub v 1 (String.length v - 1)
                               else v)));
                  }))
        { vars = []; task_ids = [] }
        (String.split_on_char '\n' text))

let env_to_text env =
  String.concat ""
    (List.map (fun (n, v) -> n ^ " = " ^ N.to_string v ^ "\n") env.vars
    @ List.map
        (fun (n, v) ->
          "$" ^ n ^ " = "
          ^ (match v with None -> "none" | Some n -> string_of_int n)
          ^ "\n")
        env.task_ids)

let get_var env name =
  Option.value ~default:N.zero (List.assoc_opt name env.vars)

let get_task_id env name =
  match List.assoc_opt name env.task_ids with
  | Some (Some id) -> id
  | Some None -> failwith ("task id $" ^ name ^ " is not initialized")
  | None -> failwith ("unknown task id $" ^ name)

let rec eval_aexp env = function
  | Atom (Var n) -> get_var env n
  | Atom (Imm n) -> n
  | Bin (a, op, b) -> (
      let a = eval_aexp env a in
      let b = eval_aexp env b in
      match op with Add -> N.add a b | Sub -> N.sub a b | Mul -> N.mul a b)

let rec eval_bexp s = function
  | Not b -> not (eval_bexp s b)
  | Rel (a, op, b) -> (
      let a = eval_aexp s.env a and b = eval_aexp s.env b in
      match op with
      | Eq -> N.equal a b
      | Ne -> not (N.equal a b)
      | Lt -> N.compare a b < 0)
  | And (a, b) -> eval_bexp s a && eval_bexp s b
  | Or (a, b) -> eval_bexp s a || eval_bexp s b
  | Done n ->
      let id = get_task_id s.env n in
      id >= 0 && id < Array.length s.tasks && s.tasks.(id) = []

let validate code =
  let seen = Hashtbl.create 16 in
  List.iter
    (fun f ->
      if Hashtbl.mem seen f.name then failwith ("duplicate function: " ^ f.name);
      Hashtbl.add seen f.name ())
    code;
  if not (Hashtbl.mem seen "main") then failwith "missing main function"

let make code agents =
  L.protect (fun () ->
      validate code;
      if agents <= 0 then failwith "agent count must be positive";
      {
        code;
        env = { vars = []; task_ids = [] };
        workers = Array.make agents None;
        queue = [ 0 ];
        tasks = [| [ { function_name = "main"; pc = 0 } ] |];
      })

let find_function code name =
  match List.find_opt (fun f -> f.name = name) code with
  | Some f -> f
  | None -> failwith ("unknown function: " ^ name)

let step s agent =
  L.protect (fun () ->
      if agent < 0 || agent >= Array.length s.workers then
        failwith (Printf.sprintf "agent %d out of range" agent);
      let workers = Array.copy s.workers and tasks = Array.copy s.tasks in
      let s = { s with workers; tasks } in
      let continue s output = Utils.Machine.Continue { next = s; output } in
      match workers.(agent) with
      | None -> (
          match s.queue with
          | [] -> Utils.Machine.Halt s.env
          | id :: queue ->
              workers.(agent) <- Some id;
              continue { s with queue }
                (Printf.sprintf "agent %d: dequeue task %d" agent id))
      | Some id -> (
          if id < 0 || id >= Array.length tasks then
            failwith (Printf.sprintf "unknown task %d" id);
          match tasks.(id) with
          | [] ->
              workers.(agent) <- None;
              continue s (Printf.sprintf "agent %d: release task %d" agent id)
          | frame :: rest ->
              let fn = find_function s.code frame.function_name in
              if frame.pc = List.length fn.body then (
                tasks.(id) <- rest;
                continue s
                  (Printf.sprintf "agent %d: return from %s" agent
                     frame.function_name))
              else
                let stmt =
                  match List.nth_opt fn.body frame.pc with
                  | Some x -> x
                  | None ->
                      failwith
                        (Printf.sprintf "pc %d out of bounds in %s" frame.pc
                           frame.function_name)
                in
                let shift offset =
                  let pc = frame.pc + offset in
                  if pc < 0 then
                    failwith
                      (Printf.sprintf "pc became negative in %s: %d + %d"
                         frame.function_name frame.pc offset);
                  tasks.(id) <- { frame with pc } :: rest
                in
                let s =
                  match stmt with
                  | Assign (n, a) ->
                      let v = eval_aexp s.env a in
                      shift 1;
                      { s with env = { s.env with vars = set s.env.vars n v } }
                  | IfGoto (b, offset) ->
                      shift (if eval_bexp s b then offset else 1);
                      s
                  | Goto offset ->
                      shift offset;
                      s
                  | Call n ->
                      ignore (find_function s.code n);
                      shift 1;
                      tasks.(id) <- { function_name = n; pc = 0 } :: tasks.(id);
                      s
                  | Run (n, id_var) ->
                      ignore (find_function s.code n);
                      let new_id = Array.length tasks in
                      shift 1;
                      {
                        s with
                        env =
                          {
                            s.env with
                            task_ids = set s.env.task_ids id_var (Some new_id);
                          };
                        tasks =
                          Array.append tasks
                            [| [ { function_name = n; pc = 0 } ] |];
                        queue = s.queue @ [ new_id ];
                      }
                  | Yield ->
                      shift 1;
                      workers.(agent) <- None;
                      { s with queue = s.queue @ [ id ] }
                in
                continue s
                  (Printf.sprintf "agent %d: %s" agent (stmt_to_text stmt))))

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      L.protect (fun () ->
          validate s.code;
          s))

let render s =
  let open Utils.Render in
  let table2 title c1 c2 rows =
    table ~title
      ~columns:[ text c1; text c2 ]
      (List.map (fun (a, b) -> [ text a; text b ]) rows)
  in
  state
    [
      table2 "status" "field" "value"
        [
          ("agents", string_of_int (Array.length s.workers));
          ("queue_len", string_of_int (List.length s.queue));
          ("task_count", string_of_int (Array.length s.tasks));
        ];
      container ~title:"env" ~orientation:"horizontal"
        [
          table2 "vars" "var" "value"
            (List.map (fun (n, v) -> (n, N.to_string v)) s.env.vars);
          table2 "task_ids" "id" "task"
            (List.map
               (fun (n, id) ->
                 ( "$" ^ n,
                   match id with None -> "none" | Some n -> string_of_int n ))
               s.env.task_ids);
        ];
      table2 "workers" "agent" "task"
        (Array.to_list
           (Array.mapi
              (fun i t ->
                ( string_of_int i,
                  match t with None -> "idle" | Some n -> string_of_int n ))
              s.workers));
      table2 "queue" "slot" "task"
        (List.mapi (fun i id -> (string_of_int i, string_of_int id)) s.queue);
      container ~title:"tasks"
        (Array.to_list
           (Array.mapi
              (fun id stack ->
                table
                  ~title:(Printf.sprintf "task %d" id)
                  ~columns:[ text "depth"; text "function"; text "pc" ]
                  (List.mapi
                     (fun i f ->
                       [
                         text (string_of_int i);
                         text f.function_name;
                         text (string_of_int f.pc);
                       ])
                     (List.rev stack)))
              s.tasks));
    ]

let routput_json text = `String text
let foutput_json env = `String (env_to_text env)

module Machine = struct
  type nonrec code = code
  type nonrec ainput = ainput
  type nonrec rinput = rinput
  type nonrec routput = routput
  type nonrec foutput = foutput
  type nonrec state = state

  let parse_code = parse_code
  let parse_ainput = parse_ainput
  let parse_rinput = parse_rinput
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
  let routput_json = routput_json
  let foutput_json = foutput_json
end
