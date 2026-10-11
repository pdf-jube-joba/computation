open Utils.Machine
module T = Turing_machine
module L = Utils.Lexer

type lvalue = Var of string | Head [@@deriving yojson]
type rvalue = Var of string | Head | Const of T.sign [@@deriving yojson]
type condition = { left : rvalue; right : rvalue } [@@deriving yojson]
type environment = (string * T.sign) list [@@deriving yojson]

let unwrap = T.unwrap
let identifier p = unwrap (Utils.Identifier.make (L.identifier p))
let create text = L.create ~identifier_extra:(( = ) '-') text
let parse f text = Utils.Parser.run ~create f text

let sign p =
  match L.next p with
  | L.Ident s -> unwrap (T.parse_sign s)
  | L.Symbol '-' -> unwrap (T.parse_sign "-")
  | _ -> L.error p "Expected sign"

let alphabet p =
  L.expect_keyword p "alphabet";
  L.expect p ":(";
  let rec loop () =
    if L.eat_symbol p ')' then []
    else
      let s = sign p in
      if L.eat_symbol p ',' then s :: loop ()
      else (
        L.expect_symbol p ')';
        [ s ])
  in
  loop ()

let lvalue p : lvalue = if L.eat_symbol p '@' then Head else Var (identifier p)

let rvalue p : rvalue =
  if L.eat_symbol p '@' then Head
  else if L.eat_keyword p "const" then Const (sign p)
  else Var (identifier p)

let condition p =
  if not (L.eat_keyword p "if") then None
  else
    let left = rvalue p in
    L.expect p "==";
    let right = rvalue p in
    Some { left; right }

let lvalue_print : lvalue -> string = function Var s -> s | Head -> "@"

let rvalue_print : rvalue -> string = function
  | Var s -> s
  | Head -> "@"
  | Const s -> "\\const " ^ T.sign_print s

let condition_print = function
  | None -> ""
  | Some c -> " \\if " ^ rvalue_print c.left ^ " == " ^ rvalue_print c.right

let alphabet_print a =
  "\\alphabet: (" ^ String.concat ", " (List.map T.sign_print a) ^ ")\n"

let vars_rvalue : rvalue -> string list = function Var s -> [ s ] | _ -> []
let vars_lvalue : lvalue -> string list = function Var s -> [ s ] | _ -> []

let vars_condition = function
  | None -> []
  | Some c -> vars_rvalue c.left @ vars_rvalue c.right

let env_create vars =
  List.map (fun s -> (s, None)) (List.sort_uniq String.compare vars)

let env_get env key = Option.value ~default:None (List.assoc_opt key env)

let env_set env key value =
  List.sort
    (fun (a, _) (b, _) -> String.compare a b)
    ((key, value) :: List.remove_assoc key env)

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
  L.protect (fun () ->
      let text = L.without_comments text in
      List.fold_left
        (fun env line ->
          let line = String.trim line in
          if line = "" then env
          else
            match String.split_on_char '=' line with
            | [ key; value ] ->
                let key = String.trim key in
                ignore (unwrap (Utils.Identifier.make key));

                env_set env key (unwrap (T.parse_sign value))
            | _ -> failwith "Invalid environment line")
        []
        (String.split_on_char '\n' text))

let environment_print env =
  String.concat "\n" (List.map (fun (k, v) -> k ^ " = " ^ T.sign_print v) env)

let render_env env =
  Utils.Render.table ~title:"env"
    ~columns:(List.map Utils.Render.text [ "var"; "value" ])
    (List.map
       (fun (k, v) -> List.map Utils.Render.text [ k; T.sign_print v ])
       env)

let parse_ainput = T.parse_tape
let parse_rinput = empty_input
let routput_json () = `String ""
let foutput_json t = `String (T.print_tape t)
