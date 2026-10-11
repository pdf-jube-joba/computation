module L = Utils.Lexer
module N = Utils.Number

let value = function Ok v -> v | Error e -> failwith e

type aatom = Var of Utils.Identifier.t | Imm of N.t [@@deriving yojson]
type abinop = Add | Sub | Mul [@@deriving yojson]
type aexp = Atom of aatom | Bin of aexp * abinop * aexp [@@deriving yojson]
type relop = Eq | Ne | Lt [@@deriving yojson]

type bexp =
  | Not of bexp
  | Rel of aexp * relop * aexp
  | And of bexp * bexp
  | Or of bexp * bexp
  | Done of Utils.Identifier.t
[@@deriving yojson]

type stmt =
  | Assign of Utils.Identifier.t * aexp
  | IfGoto of bexp * int
  | Goto of int
  | Call of Utils.Identifier.t
  | Run of Utils.Identifier.t * Utils.Identifier.t
  | Yield
[@@deriving yojson]

type fn_decl = { name : Utils.Identifier.t; body : stmt list }
[@@deriving yojson]

type code = fn_decl list [@@deriving yojson]

type global_env = {
  vars : (Utils.Identifier.t * N.t) list;
  task_ids : (Utils.Identifier.t * int option) list;
}
[@@deriving yojson]

type frame = { function_name : Utils.Identifier.t; pc : int }
[@@deriving yojson]

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

let statement_identifier = L.identifier

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

let set entries name v = Utils.Assoc.set_sorted name v entries

module Foutput = struct
  type t = foutput

  let parse_task_id text =
    if String.lowercase_ascii text = "none" then None
    else Some (value (Utils.Text.Natural_int.parse text))

  let print_task_id = function None -> "none" | Some n -> string_of_int n

  let parse text =
    Utils.Environment.fold_lines
      ~invalid_line:(fun line -> "invalid env line: " ^ line)
      (fun env name v ->
        if String.starts_with ~prefix:"$" name then (
          let name = Utils.String_ext.strip_prefix "$" name in
          ignore (value (Utils.Identifier.make name));
          let id = parse_task_id v in
          { env with task_ids = set env.task_ids name id })
        else (
          ignore (value (Utils.Identifier.make name));
          {
            env with
            vars = set env.vars name (value (N.of_string_with_plus v));
          }))
      { vars = []; task_ids = [] }
      text

  let print env =
    Utils.Environment.print ~trailing_newline:true N.to_string env.vars
    ^ Utils.Environment.print ~trailing_newline:true print_task_id
        (List.map (fun (n, v) -> ("$" ^ n, v)) env.task_ids)
end

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
  let names = List.map (fun f -> f.name) code in
  Utils.Identifier.check_unique
    ~message:(fun name -> "duplicate function: " ^ name)
    names;
  if not (List.mem "main" names) then failwith "missing main function"

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
  state
    [
      assoc_table ~title:"status" ~columns:[ "field"; "value" ] Fun.id
        [
          ("agents", string_of_int (Array.length s.workers));
          ("queue_len", string_of_int (List.length s.queue));
          ("task_count", string_of_int (Array.length s.tasks));
        ];
      container ~title:"env" ~orientation:"horizontal"
        [
          assoc_table ~title:"vars" N.to_string s.env.vars;
          assoc_table ~title:"task_ids" ~columns:[ "id"; "task" ]
            (function None -> "none" | Some n -> string_of_int n)
            (List.map (fun (n, id) -> ("$" ^ n, id)) s.env.task_ids);
        ];
      indexed_table ~title:"workers" ~columns:[ "agent"; "task" ]
        (function None -> "idle" | Some n -> string_of_int n)
        (Array.to_list s.workers);
      indexed_table ~title:"queue" ~columns:[ "slot"; "task" ] string_of_int
        s.queue;
      container ~title:"tasks"
        (Array.to_list
           (Array.mapi
              (fun id stack ->
                text_table
                  ~title:(Printf.sprintf "task %d" id)
                  ~columns:[ "depth"; "function"; "pc" ]
                  (List.mapi
                     (fun i f ->
                       [ string_of_int i; f.function_name; string_of_int f.pc ])
                     (List.rev stack)))
              s.tasks));
    ]

module Ainput = Utils.Text.Natural_int
module Rinput = Utils.Text.Natural_int
module Routput = Utils.Text.String

module Machine = struct
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec code = code
  type nonrec state = state

  let parse_code = parse_code
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
