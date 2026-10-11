open Common

type stmt =
  | Nop
  | Seq of stmt * stmt
  | Assign of Utils.Identifier.t * aexp
  | If of bexp * stmt
  | While of bexp * stmt
  | Call of
      Utils.Identifier.t * Utils.Identifier.t list * Utils.Identifier.t list
  | Return of Utils.Identifier.t list
[@@deriving yojson]

type proc_def = {
  name : Utils.Identifier.t;
  params : Utils.Identifier.t list;
  locals : Utils.Identifier.t list;
  body : stmt;
}
[@@deriving yojson]

type code = { statics : Utils.Identifier.t list; procs : proc_def list }
[@@deriving yojson]

type frame = {
  stmt : stmt;
  local_env : env;
  ret_vars : Utils.Identifier.t list;
}
[@@deriving yojson]

type state = {
  code : code;
  stmt : stmt;
  global_env : env;
  local_env : env;
  call_stack : frame list;
}
[@@deriving yojson]

type ainput = env [@@deriving yojson]
type foutput = env [@@deriving yojson]
type rinput = string
type routput = string

module Code = struct
  type t = code

  let rec parse_stmt p =
    let a = parse_single p in
    if L.eat_symbol p ';' then
      if L.peek p = L.Symbol ']' then a
      else
        let b = parse_stmt p in
        Seq (a, b)
    else a

  and parse_single p =
    if L.eat_keyword p "Nop" then Nop
    else if L.eat_keyword p "if" then (
      let c = parse_bexp p in
      L.expect_symbol p '[';
      let b = parse_stmt p in
      L.expect_symbol p ']';
      If (c, b))
    else if L.eat_keyword p "while" then (
      let c = parse_bexp p in
      L.expect_symbol p '[';
      let b = parse_stmt p in
      L.expect_symbol p ']';
      While (c, b))
    else if L.eat_keyword p "call" then (
      let n = L.identifier p in
      L.expect_symbol p '(';
      let args = names_until p ')' in
      L.expect p ") ->";
      let rets = names_boundary p in
      Call (n, args, rets))
    else if L.eat_keyword p "return" then Return (names_boundary p)
    else if L.eat_symbol p '[' then (
      let s = parse_stmt p in
      L.expect_symbol p ']';
      s)
    else
      let v = L.identifier p in
      L.expect p ":=";
      Assign (v, parse_aexp p)

  let parse =
    L.parse (fun p ->
        L.expect_keyword p "static";
        let statics = names_until p ';' in
        L.expect_symbol p ';';
        let rec procs acc =
          if L.eof p then List.rev acc
          else
            let name = L.identifier p in
            L.expect_symbol p '(';
            let params = names_until p ')' in
            L.expect p ") [ local";
            let locals = names_boundary p in
            let body = parse_stmt p in
            L.expect_symbol p ']';
            procs ({ name; params; locals; body } :: acc)
        in
        let procs = procs [] in
        if procs = [] then L.error p "At least one procedure is required";
        { statics; procs })

  let rec stmt_to_text = function
    | Nop -> "Nop"
    | Seq (a, b) -> stmt_to_text a ^ "; " ^ stmt_to_text b
    | Assign (v, a) -> v ^ " := " ^ aexp_text a
    | If (c, b) -> "if " ^ bexp_text c ^ " [" ^ stmt_to_text b ^ "]"
    | While (c, b) -> "while " ^ bexp_text c ^ " [" ^ stmt_to_text b ^ "]"
    | Call (n, args, rets) ->
        "call " ^ n ^ "(" ^ String.concat ", " args ^ ") -> "
        ^ String.concat ", " rets
    | Return vs -> "return " ^ String.concat ", " vs

  let print c =
    "static"
    ^ (if c.statics = [] then "" else " " ^ String.concat ", " c.statics)
    ^ ";\n"
    ^ String.concat ""
        (List.map
           (fun p ->
             p.name ^ "("
             ^ String.concat ", " p.params
             ^ ")[\n  local"
             ^ (if p.locals = [] then "" else " " ^ String.concat ", " p.locals)
             ^ "\n  " ^ stmt_to_text p.body ^ "\n]\n")
           c.procs)
end

let rec ends_with_return = function
  | Return _ -> true
  | Seq (_, b) -> ends_with_return b
  | _ -> false

let with_implicit_return s = if ends_with_return s then s else Seq (s, Return [])

let lookup_var s v =
  match List.assoc_opt v s.local_env with
  | Some n -> n
  | None -> Utils.Assoc.lookup v s.global_env ("Unknown variable: " ^ v)

let assign_var s v n =
  if List.mem_assoc v s.local_env then
    { s with local_env = Utils.Assoc.set v n s.local_env }
  else if List.mem_assoc v s.global_env then
    { s with global_env = Utils.Assoc.set v n s.global_env }
  else failwith ("Unknown variable: " ^ v)

let call_into s name args rets continuation =
  let p =
    match List.find_opt (fun p -> p.name = name) s.code.procs with
    | Some p -> p
    | None -> failwith ("Unknown procedure: " ^ name)
  in
  if List.length args <> List.length p.params then
    failwith
      (Printf.sprintf "Argument count mismatch for %s: expected %d, got %d" name
         (List.length p.params) (List.length args));
  let values = List.map (lookup_var s) args in
  let local_env =
    List.fold_left
      (fun e v -> Utils.Assoc.set v N.zero e)
      (List.combine p.params values)
      p.locals
    |> Utils.Assoc.sorted
  in
  ( with_implicit_return p.body,
    {
      s with
      local_env;
      call_stack =
        { stmt = continuation; local_env = s.local_env; ret_vars = rets }
        :: s.call_stack;
    } )

let rec step_stmt s = function
  | Nop -> (Nop, s)
  | Assign (v, a) -> (Nop, assign_var s v (eval_aexp (lookup_var s) a))
  | If (c, b) -> ((if eval_bexp (lookup_var s) c then b else Nop), s)
  | While (c, b) -> (If (c, Seq (b, While (c, b))), s)
  | Seq (Nop, b) -> (b, s)
  | Seq (Return vs, _) -> (Return vs, s)
  | Seq (Call (n, args, rets), b) -> call_into s n args rets b
  | Seq (a, b) ->
      let a, s = step_stmt s a in
      (Seq (a, b), s)
  | Call (n, args, rets) -> call_into s n args rets Nop
  | Return vs -> (
      match s.call_stack with
      | [] -> (Return vs, s)
      | frame :: call_stack ->
          if List.length vs <> List.length frame.ret_vars then
            failwith
              (Printf.sprintf "Return count mismatch: expected %d, got %d"
                 (List.length frame.ret_vars)
                 (List.length vs));
          let values = List.map (lookup_var s) vs in
          let s = { s with call_stack; local_env = frame.local_env } in
          ( frame.stmt,
            List.fold_left
              (fun s (v, n) -> assign_var s v n)
              s
              (List.combine frame.ret_vars values) ))

let validate code =
  Utils.Identifier.check_unique
    ~message:(fun n -> "Duplicate static variable: " ^ n)
    code.statics;
  Utils.Identifier.check_unique
    ~message:(fun n -> "Duplicate procedure name: " ^ n)
    (List.map (fun p -> p.name) code.procs);
  List.iter
    (fun p ->
      let seen = ref [] in
      List.iter
        (fun v ->
          if List.mem v code.statics then
            failwith
              ("Name conflict in " ^ p.name ^ ": parameter " ^ v
             ^ " conflicts with static");
          if List.mem v !seen then
            failwith ("Duplicate parameter " ^ v ^ " in " ^ p.name);
          seen := v :: !seen)
        p.params;
      List.iter
        (fun v ->
          if List.mem v code.statics then
            failwith
              ("Name conflict in " ^ p.name ^ ": local " ^ v
             ^ " conflicts with static");
          if List.mem v !seen then
            failwith
              ("Name conflict in " ^ p.name ^ ": local " ^ v
             ^ " conflicts with parameter/local");
          seen := v :: !seen)
        p.locals)
    code.procs;
  let main =
    match List.find_opt (fun p -> p.name = "main") code.procs with
    | Some p -> p
    | None -> failwith "Procedure 'main' is required"
  in
  if main.params <> [] then failwith "main must have no parameters";
  main

let make code global_env =
  L.protect (fun () ->
      let main = validate code in
      validate_env code.statics global_env
        ~duplicate:(fun n -> "Duplicate static variable: " ^ n)
        ~extra:(fun n -> "AInput contains undeclared static variable: " ^ n)
        ~missing:(fun n -> "AInput missing static variable: " ^ n);
      {
        code;
        stmt = with_implicit_return main.body;
        global_env = Utils.Assoc.sorted global_env;
        local_env =
          Utils.Assoc.sorted (List.map (fun v -> (v, N.zero)) main.locals);
        call_stack = [];
      })

let step s _ =
  L.protect (fun () ->
      match (s.stmt, s.call_stack) with
      | Return _, [] -> Utils.Machine.Halt s.global_env
      | _ ->
          let output = Code.stmt_to_text s.stmt in
          let stmt, next = step_stmt s s.stmt in
          Utils.Machine.Continue { next = { next with stmt }; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.state
    [
      Utils.Render.text ~title:"stmt" (Code.stmt_to_text s.stmt);
      Utils.Render.text ~title:"global env"
        (Env.print_inline (Utils.Assoc.sorted s.global_env));
      Utils.Render.text ~title:"local env"
        (Env.print_inline (Utils.Assoc.sorted s.local_env));
      Utils.Render.text ~title:"call stack depth"
        (string_of_int (List.length s.call_stack));
    ]

module Ainput = struct
  type t = ainput

  let parse text = Result.map Utils.Assoc.sorted (Env.parse text)
  let print = fun env -> Env.print (Utils.Assoc.sorted env)
end

module Rinput = Utils.Text.String
module Routput = Utils.Text.String
module Foutput = Ainput

module Machine = struct
  module Code = Code
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec state = state

  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
