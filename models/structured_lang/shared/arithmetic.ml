module N = Utils_base.Number
module L = Utils_parsing.Parser

let get = Utils_base.Error.get

type env = (Utils_names.Identifier.t * N.t) list [@@deriving yojson]

let env_get e k = Option.value (List.assoc_opt k e) ~default:N.zero

module Env = struct
  type t = env

  let parse = Utils_codec.Environment_text.parse N.of_string

  let print =
    Utils_codec.Environment_text.print ~trailing_newline:true N.to_string

  let print_inline = Utils_codec.Environment_text.print N.to_string
end

let names_until p endc =
  if L.peek p = L.Symbol endc then []
  else L.separated p ~separator:',' L.identifier

let names_boundary p =
  match L.peek p with L.Ident _ -> names_until p '\000' | _ -> []

let validate_env statics env ~duplicate ~extra ~missing =
  Utils_names.Identifier.check_unique ~message:duplicate statics;
  List.iter
    (fun (k, _) -> if not (List.mem k statics) then failwith (extra k))
    env;
  List.iter
    (fun k -> if not (List.mem_assoc k env) then failwith (missing k))
    statics

type atom = Var of Utils_names.Identifier.t | Imm of N.t [@@deriving yojson]
type abinop = Add | Sub [@@deriving yojson]
type aexp = Atom of atom | Bin of atom * abinop * atom [@@deriving yojson]
type relop = Lt | Eq | Gt [@@deriving yojson]
type bexp = atom * relop * atom [@@deriving yojson]

let parse_atom p =
  match L.peek p with
  | L.Number _ -> Imm (L.natural p)
  | _ -> Var (L.identifier p)

let parse_aexp p =
  let a = parse_atom p in
  if L.bump_if_symbol p '+' then Bin (a, Add, parse_atom p)
  else if L.bump_if_symbol p '-' then Bin (a, Sub, parse_atom p)
  else Atom a

let parse_bexp p =
  L.binary p
    ~operators:[ ("<", Lt); ("=", Eq); (">", Gt) ]
    ~operand:parse_atom
    ~make:(fun a op b -> (a, op, b))

let atom_text = function
  | Var v -> Utils_names.Identifier.to_string v
  | Imm n -> N.to_string n

let aexp_text = function
  | Atom a -> atom_text a
  | Bin (a, op, b) ->
      atom_text a ^ (if op = Add then " + " else " - ") ^ atom_text b

let rel_text = function Lt -> "<" | Eq -> "=" | Gt -> ">"
let bexp_text (a, op, b) = atom_text a ^ " " ^ rel_text op ^ " " ^ atom_text b
let eval_atom lookup = function Var v -> lookup v | Imm n -> n

let eval_aexp lookup = function
  | Atom a -> eval_atom lookup a
  | Bin (a, op, b) ->
      (if op = Add then N.add else N.sub)
        (eval_atom lookup a) (eval_atom lookup b)

let eval_bexp lookup (a, op, b) =
  let c = N.compare (eval_atom lookup a) (eval_atom lookup b) in
  match op with Lt -> c < 0 | Eq -> c = 0 | Gt -> c > 0

let atom_vars = function Var v -> [ v ] | Imm _ -> []

let aexp_vars = function
  | Atom a -> atom_vars a
  | Bin (a, _, b) -> atom_vars a @ atom_vars b

let bexp_vars (a, _, b) = atom_vars a @ atom_vars b
