module T = Turing_machine.Main
module L = Utils.Parser

type lvalue = Var of Utils.Identifier.t | Head [@@deriving yojson]

type rvalue = Var of Utils.Identifier.t | Head | Const of T.sign
[@@deriving yojson]

type condition = { left : rvalue; right : rvalue } [@@deriving yojson]
type environment = (Utils.Identifier.t * T.sign) list [@@deriving yojson]

let keywords =
  [
    "alphabet";
    "fn";
    "label";
    "jump";
    "call";
    "const";
    "return";
    "break";
    "continue";
    "if";
    "LT";
    "RT";
  ]

let identifier ?(keywords = keywords) p =
  let s = L.identifier p in
  if List.mem s keywords then L.error p ("identifier cannot be a keyword: " ^ s);
  s

let sign p =
  match L.bump p with
  | L.Symbol '-' -> None
  | L.Ident s -> T.unwrap (T.Sign.parse s)
  | _ -> L.error p "Expected sign"

let alphabet p =
  L.expect p "alphabet:(";
  L.separated_until ~allow_trailing:true p ~separator:',' ~closing:')' sign

let lvalue ?keywords p : lvalue =
  if L.bump_if_symbol p '@' then Head else Var (identifier ?keywords p)

let rvalue ?keywords p : rvalue =
  if L.bump_if_symbol p '@' then Head
  else if L.bump_if_keyword p "const" then Const (sign p)
  else Var (identifier ?keywords p)

let condition ?keywords p =
  if not (L.bump_if_keyword p "if") then None
  else
    let left = rvalue ?keywords p in
    L.expect p "==";
    let right = rvalue ?keywords p in
    Some { left; right }

let lvalue_print : lvalue -> string = function Var s -> s | Head -> "@"

let rvalue_print : rvalue -> string = function
  | Var s -> s
  | Head -> "@"
  | Const s -> "const " ^ T.Sign.print s

let condition_print = function
  | None -> ""
  | Some c -> " if " ^ rvalue_print c.left ^ " == " ^ rvalue_print c.right

let alphabet_print a =
  "alphabet: (" ^ String.concat ", " (List.map T.Sign.print a) ^ ")\n"

let vars_rvalue : rvalue -> string list = function Var s -> [ s ] | _ -> []
let vars_lvalue : lvalue -> string list = function Var s -> [ s ] | _ -> []

let vars_condition = function
  | None -> []
  | Some c -> vars_rvalue c.left @ vars_rvalue c.right

let env_create vars =
  List.map (fun s -> (s, None)) (List.sort_uniq String.compare vars)

let eval env tape : rvalue -> T.sign = function
  | Var s -> Option.value ~default:None (List.assoc_opt s env)
  | Head -> tape.T.head
  | Const s -> s

let eval_cond env tape = function
  | None -> true
  | Some c -> eval env tape c.left = eval env tape c.right

let assign env tape (dst : lvalue) value =
  match dst with
  | Var s -> (Utils.Assoc.set_sorted s value env, tape)
  | Head -> (env, T.Tape.head_write tape value)

let normalized_alphabet a =
  if a = [] then failwith "alphabet must not be empty";
  let a =
    List.fold_left (fun xs s -> if List.mem s xs then xs else xs @ [ s ]) [] a
  in
  if List.mem None a then a else a @ [ None ]

let validate_rvalue allowed : rvalue -> unit = function
  | Const s when not (List.mem s allowed) ->
      failwith ("Unknown sign in const: " ^ T.Sign.print s)
  | _ -> ()

let validate_cond allowed = function
  | None -> ()
  | Some c ->
      validate_rvalue allowed c.left;
      validate_rvalue allowed c.right

let validate_tape allowed t =
  List.iter
    (fun s ->
      if not (List.mem s allowed) then
        failwith ("Unknown sign on tape: " ^ T.Sign.print s))
    (fst (T.Tape.into_vec t))

module Env = struct
  type t = (Utils.Identifier.t * T.sign) list

  let parse_entry env key value =
    if String.contains value '=' then failwith "Invalid environment line";
    ignore (T.unwrap (Utils.Identifier.make key));
    if List.mem key keywords then failwith "Reserved environment variable";
    Utils.Assoc.set_sorted key (T.unwrap (T.Sign.parse value)) env

  let parse text =
    Utils.Environment.fold_lines
      ~invalid_line:(fun _ -> "Invalid environment line")
      parse_entry [] text

  let print = Utils.Environment.print T.Sign.print
end

module Tape = T.Ainput
