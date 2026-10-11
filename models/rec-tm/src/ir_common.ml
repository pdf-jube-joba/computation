open Utils.Machine
module T = Turing_machine
module L = Utils.Lexer

type lvalue = Var of Utils.Identifier.t | Head [@@deriving yojson]

type rvalue = Var of Utils.Identifier.t | Head | Const of T.sign
[@@deriving yojson]

type condition = { left : rvalue; right : rvalue } [@@deriving yojson]
type environment = (Utils.Identifier.t * T.sign) list [@@deriving yojson]

let unwrap = T.unwrap

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
  unwrap (Utils.Identifier.make s)

(* The IR has a restricted set of symbols and uses '-' for the blank sign. *)
let create text =
  let n = String.length text in
  let alpha = Utils.Identifier.is_start in
  let digit = function '0' .. '9' -> true | _ -> false in
  let rec scan i acc =
    if i = n then Array.of_list (List.rev ((L.Eof, n) :: acc))
    else
      match text.[i] with
      | c when Utils.String_ext.is_whitespace c -> scan (i + 1) acc
      | c when alpha c || digit c ->
          let predicate =
            if digit c then digit else Utils.Identifier.is_continue
          in
          let j = Utils.String_ext.skip_while predicate text (i + 1) in
          let s = String.sub text i (j - i) in
          scan j (((if digit c then L.Number s else L.Ident s), i) :: acc)
      | '-' -> scan (i + 1) ((L.Symbol '-', i) :: acc)
      | ('(' | ')' | '{' | '}' | ',' | '@' | ':' | '=') as c ->
          scan (i + 1) ((L.Symbol c, i) :: acc)
      | c -> failwith (Printf.sprintf "Unexpected character '%c'" c)
  in
  { L.tokens = scan 0 []; pos = 0 }

let parse f text = L.parse_with create f text

let sign p =
  match L.next p with
  | L.Symbol '-' -> None
  | L.Ident s -> unwrap (T.parse_sign s)
  | _ -> L.error p "Expected sign"

let alphabet p =
  L.expect p "alphabet:(";
  L.separated_until ~allow_trailing:true p ~separator:',' ~closing:')' sign

let lvalue ?keywords p : lvalue =
  if L.eat_symbol p '@' then Head else Var (identifier ?keywords p)

let rvalue ?keywords p : rvalue =
  if L.eat_symbol p '@' then Head
  else if L.eat_keyword p "const" then Const (sign p)
  else Var (identifier ?keywords p)

let condition ?keywords p =
  if not (L.eat_keyword p "if") then None
  else
    let left = rvalue ?keywords p in
    L.expect p "==";
    let right = rvalue ?keywords p in
    Some { left; right }

let lvalue_print : lvalue -> string = function Var s -> s | Head -> "@"

let rvalue_print : rvalue -> string = function
  | Var s -> s
  | Head -> "@"
  | Const s -> "const " ^ T.sign_print s

let condition_print = function
  | None -> ""
  | Some c -> " if " ^ rvalue_print c.left ^ " == " ^ rvalue_print c.right

let alphabet_print a =
  "alphabet: (" ^ String.concat ", " (List.map T.sign_print a) ^ ")\n"

let vars_rvalue : rvalue -> string list = function Var s -> [ s ] | _ -> []
let vars_lvalue : lvalue -> string list = function Var s -> [ s ] | _ -> []

let vars_condition = function
  | None -> []
  | Some c -> vars_rvalue c.left @ vars_rvalue c.right

let env_create vars =
  List.map (fun s -> (s, None)) (List.sort_uniq String.compare vars)

let env_get env key = Option.value ~default:None (List.assoc_opt key env)
let env_set env key value = Utils.Assoc.set_sorted key value env

let eval env tape : rvalue -> T.sign = function
  | Var s -> env_get env s
  | Head -> tape.T.head
  | Const s -> s

let eval_cond env tape = function
  | None -> true
  | Some c -> eval env tape c.left = eval env tape c.right

let assign env tape (dst : lvalue) value =
  match dst with
  | Var s -> (env_set env s value, tape)
  | Head -> (env, T.head_write tape value)

let normalized_alphabet a =
  if a = [] then failwith "alphabet must not be empty";
  let a =
    List.fold_left (fun xs s -> if List.mem s xs then xs else xs @ [ s ]) [] a
  in
  if List.mem None a then a else a @ [ None ]

let validate_rvalue allowed : rvalue -> unit = function
  | Const s when not (List.mem s allowed) ->
      failwith ("Unknown sign in const: " ^ T.sign_print s)
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
        failwith ("Unknown sign on tape: " ^ T.sign_print s))
    (fst (T.tape_into_vec t))

let environment_parse text =
  Utils.Environment.fold_lines
    ~invalid_line:(fun _ -> "Invalid environment line")
    (fun env key value ->
      if String.contains value '=' then failwith "Invalid environment line";
      ignore (unwrap (Utils.Identifier.make key));
      if List.mem key keywords then failwith "Reserved environment variable";
      env_set env key (unwrap (T.parse_sign value)))
    [] text

let environment_print = Utils.Environment.print T.sign_print
let render_env env = Utils.Render.assoc_table ~title:"env" T.sign_print env
let parse_ainput = T.parse_tape
let parse_rinput = empty_input
let routput_json () = `String ""
let foutput_json t = `String (T.print_tape t)
