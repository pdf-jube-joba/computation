open Structured_lang_shared.Arithmetic

type stmt =
  | Nop
  | Seq of stmt * stmt
  | Assign of Utils.Identifier.t * aexp
  | If of bexp * stmt
  | Break of Utils.Identifier.t * Utils.Identifier.t
  | Continue of Utils.Identifier.t
  | Loop of Utils.Identifier.t * stmt * Utils.Identifier.t
[@@deriving yojson]

type code = { statics : Utils.Identifier.t list; body : stmt }
[@@deriving yojson]

type control =
  | Normal
  | Break_result of Utils.Identifier.t * N.t
  | Continue_result of Utils.Identifier.t
[@@deriving yojson]

type exec = Stmt of stmt | Result of control [@@deriving yojson]

type frame =
  | Loop_frame of Utils.Identifier.t * stmt * Utils.Identifier.t
  | Seq_frame of stmt
[@@deriving yojson]

type ainput = env [@@deriving yojson]
type foutput = env [@@deriving yojson]
type rinput = unit
type routput = string

type state = { code : code; env : env; exec : exec; stack : frame list }
[@@deriving yojson]

module Code = struct
  type t = code

  let label p =
    L.expect_symbol p ':';
    L.identifier p

  let rec parse_stmt p =
    let a = parse_single p in
    let rec loop a =
      if L.eat_symbol p ';' then
        if L.eof p || L.peek p = L.Symbol '}' then a
        else
          let b = parse_single p in
          loop (Seq (a, b))
      else a
    in
    loop a

  and parse_block p =
    L.expect_symbol p '{';
    let s = parse_stmt p in
    L.expect_symbol p '}';
    s

  and parse_single p =
    if L.eat_keyword p "Nop" then Nop
    else if L.eat_keyword p "if" then
      let c = parse_bexp p in
      let b = parse_block p in
      If (c, b)
    else if L.eat_keyword p "break" then
      let l = label p in
      let v = L.identifier p in
      Break (l, v)
    else if L.eat_keyword p "continue" then Continue (label p)
    else if L.eat_keyword p "loop" then (
      let l = label p in
      L.expect_symbol p '(';
      let b = parse_stmt p in
      L.expect p ") ->";
      let v = L.identifier p in
      Loop (l, b, v))
    else if L.peek p = L.Symbol '{' then parse_block p
    else
      let v = L.identifier p in
      L.expect p ":=";
      Assign (v, parse_aexp p)

  let parse =
    L.parse (fun p ->
        L.expect_keyword p "static";
        let statics = names_until p ';' in
        L.expect_symbol p ';';
        let body = parse_stmt p in
        { statics; body })

  let rec stmt_to_text = function
    | Nop -> "Nop"
    | Seq (a, b) -> stmt_to_text a ^ "; " ^ stmt_to_text b
    | Assign (v, a) -> v ^ " := " ^ aexp_text a
    | If (c, b) -> "if " ^ bexp_text c ^ " { " ^ stmt_to_text b ^ " }"
    | Break (l, v) -> "break :" ^ l ^ " " ^ v
    | Continue l -> "continue :" ^ l
    | Loop (l, b, v) -> "loop :" ^ l ^ " (" ^ stmt_to_text b ^ ") -> " ^ v

  let print c =
    "static"
    ^ (if c.statics = [] then "" else " " ^ String.concat ", " c.statics)
    ^ "; " ^ stmt_to_text c.body ^ "\n"
end

let exec_to_text = function
  | Stmt s -> Code.stmt_to_text s
  | Result Normal -> "()"
  | Result (Break_result (l, v)) -> "break " ^ l ^ " " ^ N.to_string v
  | Result (Continue_result l) -> "continue " ^ l

let make code env =
  L.protect (fun () ->
      validate_env code.statics env
        ~duplicate:(fun s -> "duplicate static variable: " ^ s)
        ~extra:(fun s -> "ainput contains undeclared static variable: " ^ s)
        ~missing:(fun s -> "ainput missing static variable: " ^ s);
      { code; env; exec = Stmt code.body; stack = [] })

let step s () =
  L.protect (fun () ->
      let output = exec_to_text s.exec in
      let next =
        match s.exec with
        | Stmt Nop -> { s with exec = Result Normal }
        | Stmt (Seq (a, b)) ->
            { s with exec = Stmt a; stack = Seq_frame b :: s.stack }
        | Stmt (Assign (v, a)) ->
            {
              s with
              env = Utils.Assoc.set v (eval_aexp (env_get s.env) a) s.env;
              exec = Result Normal;
            }
        | Stmt (If (c, b)) ->
            {
              s with
              exec =
                (if eval_bexp (env_get s.env) c then Stmt b else Result Normal);
            }
        | Stmt (Break (l, v)) ->
            { s with exec = Result (Break_result (l, env_get s.env v)) }
        | Stmt (Continue l) -> { s with exec = Result (Continue_result l) }
        | Stmt (Loop (l, b, v)) ->
            {
              s with
              stack = Loop_frame (l, b, v) :: s.stack;
              exec = Result Normal;
            }
        | Result control -> (
            match s.stack with
            | [] -> (
                match control with
                | Normal -> s
                | Break_result (l, _) -> failwith ("break outside loop: " ^ l)
                | Continue_result l -> failwith ("continue outside loop: " ^ l))
            | Seq_frame b :: stack ->
                {
                  s with
                  stack;
                  exec =
                    (match control with
                    | Normal -> Stmt b
                    | _ -> Result control);
                }
            | Loop_frame (l, b, v) :: stack -> (
                match control with
                | Normal -> { s with exec = Stmt b }
                | Continue_result got when got = l ->
                    { s with exec = Result Normal }
                | Break_result (got, n) when got = l ->
                    {
                      s with
                      stack;
                      exec = Result Normal;
                      env = Utils.Assoc.set v n s.env;
                    }
                | _ -> { s with stack; exec = Result control }))
      in
      match (s.exec, next.exec, next.stack) with
      | Result _, Result Normal, [] -> Utils.Machine.Halt next.env
      | _ -> Utils.Machine.Continue { next; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.state
    [
      Utils.Render.text ~title:"exec" (exec_to_text s.exec);
      Utils.Render.text ~title:"env" (Env.print_inline s.env);
      Utils.Render.text ~title:"stack"
        (String.concat "\n"
           (List.map
              (function
                | Loop_frame (l, _, v) -> "loop " ^ l ^ " -> " ^ v
                | Seq_frame b -> "seq " ^ Code.stmt_to_text b)
              (List.rev s.stack)));
    ]

module Ainput = Structured_lang_shared.Arithmetic.Env
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.String
module Foutput = Ainput
