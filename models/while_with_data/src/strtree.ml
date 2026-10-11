open Data_parser

type aexp = Mtoa of texp | AtomChar of string

and texp =
  | Var of Utils.Identifier.t
  | Atom of aexp
  | Cons of texp * texp
  | Left of texp
  | Right of texp
[@@deriving yojson]

type bexp = IsAtom of texp | Eq of texp * texp [@@deriving yojson]

type stmt =
  | Nop
  | Assign of Utils.Identifier.t * texp
  | Seq of stmt * stmt
  | IfEq of bexp * stmt
  | While of bexp * stmt
[@@deriving yojson]

type value = AtomValue of string | ConsValue of value * value
[@@deriving yojson]

type code = stmt [@@deriving yojson]
type environment = (Utils.Identifier.t * value) list [@@deriving yojson]
type ainput = environment
type rinput = unit
type routput = unit
type foutput = environment
type state = { code : code; stmt : stmt; env : environment } [@@deriving yojson]

let rec aexp_to_text = function
  | Mtoa t -> "mtoa " ^ texp_to_text t
  | AtomChar c -> char_text c

and texp_to_text = function
  | Var n -> n
  | Atom a -> "atom " ^ aexp_to_text a
  | Cons (a, b) -> "cons " ^ texp_to_text a ^ " " ^ texp_to_text b
  | Left a -> "left " ^ texp_to_text a
  | Right a -> "right " ^ texp_to_text a

let bexp_to_text = function
  | IsAtom a -> "is_atom " ^ texp_to_text a
  | Eq (a, b) -> "(" ^ texp_to_text a ^ " == " ^ texp_to_text b ^ ")"

let rec stmt_to_text = function
  | Nop -> "Nop"
  | Assign (n, a) -> n ^ " := " ^ texp_to_text a
  | Seq (a, b) -> stmt_to_text a ^ " ; " ^ stmt_to_text b
  | IfEq (c, b) -> "ifeq " ^ bexp_to_text c ^ " then " ^ stmt_to_text b ^ " end"
  | While (c, b) -> "while " ^ bexp_to_text c ^ " { " ^ stmt_to_text b ^ " }"

let print = stmt_to_text

let rec value_to_text = function
  | AtomValue c -> "atom " ^ char_text c
  | ConsValue (a, b) -> "cons " ^ value_to_text a ^ " " ^ value_to_text b

let reserved =
  [
    "Nop";
    "ifeq";
    "then";
    "end";
    "while";
    "is_atom";
    "mtoa";
    "atom";
    "cons";
    "left";
    "right";
  ]

let parse_name p =
  let n = ident p in
  if List.mem n reserved then fail "Expected identifier";
  n

let rec parse_aexp p =
  if eat p (Word "mtoa") then Mtoa (parse_texp p)
  else if eat p (Sym '(') then (
    let a = parse_aexp p in
    expect p (Sym ')');
    a)
  else AtomChar (character p)

and parse_texp p =
  if eat p (Word "atom") then Atom (parse_aexp p)
  else if eat p (Word "cons") then
    let a = parse_texp p in
    let b = parse_texp p in
    Cons (a, b)
  else if eat p (Word "left") then Left (parse_texp p)
  else if eat p (Word "right") then Right (parse_texp p)
  else if eat p (Sym '(') then (
    let a = parse_texp p in
    expect p (Sym ')');
    a)
  else Var (parse_name p)

let parse_bexp p =
  if eat p (Word "is_atom") then IsAtom (parse_texp p)
  else
    (* Rust's printer parenthesizes equality. Accept that printed form too. *)
    let saved = p.pos in
    let relation () =
      let a = parse_texp p in
      expect p (Op "==");
      let b = parse_texp p in
      Eq (a, b)
    in
    try relation ()
    with Failure _ ->
      p.pos <- saved;
      expect p (Sym '(');
      let c = relation () in
      expect p (Sym ')');
      c

let rec parse_stmt p =
  let s = parse_single p in
  let rec loop s =
    if eat p (Sym ';') then
      let b = parse_single p in
      loop (Seq (s, b))
    else s
  in
  loop s

and parse_single p =
  if eat p (Word "Nop") then Nop
  else if eat p (Word "ifeq") then (
    let c = parse_bexp p in
    expect p (Word "then");
    let b = parse_stmt p in
    expect p (Word "end");
    IfEq (c, b))
  else if eat p (Word "while") then (
    let c = parse_bexp p in
    expect p (Sym '{');
    let b = parse_stmt p in
    expect p (Sym '}');
    While (c, b))
  else if eat p (Sym '(') then (
    let b = parse_stmt p in
    expect p (Sym ')');
    b)
  else
    let n = parse_name p in
    expect p (Op ":=");
    let a = parse_texp p in
    Assign (n, a)

let parse_code = parse parse_stmt

let rec lower_value = function
  | Atom (AtomChar c) -> AtomValue c
  | Cons (a, b) -> ConsValue (lower_value a, lower_value b)
  | Var n -> fail ("unexpected variable in value: " ^ n)
  | Atom (Mtoa _) -> fail "unexpected mtoa in value"
  | Left _ | Right _ -> fail "unexpected projection in value"

let parse_value = parse (fun p -> lower_value (parse_texp p))
let parse_ainput = env parse_value
let parse_rinput = Utils.Machine.empty_input

let rec eval_aexp env = function
  | AtomChar c -> c
  | Mtoa t -> (
      match eval_texp env t with
      | AtomValue c -> c
      | _ -> fail "mtoa expects an atom")

and eval_texp env = function
  | Var n -> (
      match List.assoc_opt n env with
      | Some v -> v
      | None -> fail ("uninitialized variable: " ^ n))
  | Atom a -> AtomValue (eval_aexp env a)
  | Cons (a, b) ->
      let a = eval_texp env a in
      let b = eval_texp env b in
      ConsValue (a, b)
  | Left t -> (
      match eval_texp env t with
      | ConsValue (a, _) -> a
      | _ -> fail "left expects a cons cell")
  | Right t -> (
      match eval_texp env t with
      | ConsValue (_, b) -> b
      | _ -> fail "right expects a cons cell")

let eval_bexp env = function
  | IsAtom t -> ( match eval_texp env t with AtomValue _ -> true | _ -> false)
  | Eq (a, b) ->
      let a = eval_texp env a in
      let b = eval_texp env b in
      a = b

let rec small_step stmt env =
  match stmt with
  | Nop -> (Nop, env)
  | Assign (n, a) -> (Nop, set env n (eval_texp env a))
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | IfEq (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (IfEq (c, Seq (b, While (c, b))), env)

let make code env = Ok { code; stmt = code; env }

let step s () =
  Utils.Lexer.protect (fun () ->
      if s.stmt = Nop then Utils.Machine.Halt s.env
      else
        let stmt, env = small_step s.stmt s.env in
        Utils.Machine.Continue { next = { s with stmt; env }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.statement_env (stmt_to_text s.stmt) value_to_text s.env

let routput_json () = `String ""
let foutput_json env = `String (env_text value_to_text env)

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
