open Syntax
open Utils_runtime.Machine

type value = Addr of int | Int of N.t [@@deriving yojson]
type env = (Id.t * value) list [@@deriving yojson]

type result = Number of N.t | Data of Id.t * value list | Function of int
[@@deriving yojson]

type closure =
  | Closure of lambda * env
  | Blackhole
  | Evaluated of result
  | Pap of int * value list
[@@deriving yojson]

type control = Eval of expr * env | Enter of value | Return of result
[@@deriving yojson]

(* The top of each stack is the head of the list. Update frames delimit the
   return stack as well as saving the argument stack. *)
type frame =
  | Match of alternative list * env * value list
  | Update of int * value list
[@@deriving yojson]

type state = {
  code : Syntax.t;
  globals : env;
  heap : (int * closure) list;
  next_addr : int;
  control : control;
  arguments : value list;
  stack : frame list;
}
[@@deriving yojson]

module Code = Syntax
module Ainput = Utils_data.Text.Unit
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit

let value_text = function
  | Addr a -> "@" ^ string_of_int a
  | Int n -> N.to_string n

let values_text xs = String.concat ", " (List.map value_text xs)

let result_text = function
  | Number n -> N.to_string n
  | Data (c, xs) -> "\\con " ^ Id.to_string c ^ "(" ^ values_text xs ^ ")"
  | Function a -> "\\function @" ^ string_of_int a

module Foutput = struct
  type t = result

  let print = result_text

  let read_value p =
    if P.bump_if_symbol p '@' then Addr (P.integer p) else Int (P.natural p)

  let parse =
    P.parse (fun p ->
        match P.peek p with
        | P.Number _ -> Number (P.natural p)
        | P.Keyword "con" ->
            P.expect_keyword p "con";
            let c = P.identifier p in
            Data (c, list p P.Parentheses read_value)
        | P.Keyword "function" ->
            P.expect_keyword p "function";
            P.expect_symbol p '@';
            Function (P.integer p)
        | _ -> P.error p "Expected a natural, constructor or function result")
end

let lookup env x =
  match List.assoc_opt x env with
  | Some v -> v
  | None -> failwith ("Unbound variable: " ^ Id.to_string x)

let lookup_heap s a =
  match List.assoc_opt a s.heap with
  | Some c -> c
  | None -> failwith ("Invalid heap address: @" ^ string_of_int a)

let bind bindings env =
  bindings @ List.filter (fun (x, _) -> not (List.mem_assoc x bindings)) env

let atom env = function Var x -> lookup env x | Lit n -> Int n

let set_heap s a closure =
  {
    s with
    heap = List.map (fun (b, old) -> (b, if a = b then closure else old)) s.heap;
  }

let allocate s closure =
  if s.next_addr = max_int then failwith "Heap address space exhausted";
  let a = s.next_addr in
  (a, { s with heap = (a, closure) :: s.heap; next_addr = a + 1 })

let allocate_bindings s env recursive bindings =
  let count = List.length bindings in
  if count > max_int - s.next_addr then failwith "Heap address space exhausted";
  let addresses =
    List.mapi (fun i (x, _) -> (x, Addr (s.next_addr + i))) bindings
  in
  let inner = bind addresses env in
  let captures = (if recursive then inner else env) @ s.globals in
  let heap =
    List.mapi
      (fun i (_, lf) ->
        ( s.next_addr + i,
          Closure (lf, List.map (fun x -> (x, lookup captures x)) lf.free) ))
      bindings
  in
  ({ s with heap = heap @ s.heap; next_addr = s.next_addr + count }, inner)

let make code () =
  let* () = Code.validate code in
  let globals = List.mapi (fun a (x, _) -> (x, Addr a)) code in
  Ok
    {
      code;
      globals;
      heap = List.mapi (fun a (_, lf) -> (a, Closure (lf, []))) code;
      next_addr = List.length code;
      control = Enter (lookup globals (Id.make_exn "main"));
      arguments = [];
      stack = [];
    }

let rec take n xs =
  if n = 0 then ([], xs)
  else
    match xs with
    | x :: xs ->
        let prefix, rest = take (n - 1) xs in
        (x :: prefix, rest)
    | [] -> failwith "Insufficient arguments"

let primitive op a b =
  match (a, b) with
  | Int a, Int b ->
      let boolean b = if b then N.one else N.zero in
      Number
        (match op with
        | Add -> N.add a b
        | Sub -> N.sub a b
        | Mul -> N.mul a b
        | Eq -> boolean (N.equal a b)
        | Lt -> boolean (N.compare a b < 0))
  | _ ->
      failwith
        "Primitive operands must be unboxed naturals; force them with case"

let match_result s value alts env =
  let matches = function
    | Constructor (c, _), _ -> (
        match value with Data (d, _) -> Id.equal c d | _ -> false)
    | Literal n, _ -> ( match value with Number m -> N.equal n m | _ -> false)
    | Default _, _ -> false
  in
  let chosen =
    match List.find_opt matches alts with
    | Some alt -> Some alt
    | None -> List.find_opt (function Default _, _ -> true | _ -> false) alts
  in
  match chosen with
  | None -> failwith "No matching case alternative"
  | Some (Constructor (_, names), body) -> (
      match value with
      | Data (_, fields) when List.length names = List.length fields ->
          (s, body, bind (List.combine names fields) env)
      | _ -> failwith "Constructor pattern arity mismatch")
  | Some (Literal _, body) | Some (Default None, body) -> (s, body, env)
  | Some (Default (Some x), body) ->
      let v, s =
        match value with
        | Number n -> (Int n, s)
        | Function a -> (Addr a, s)
        | Data _ ->
            let a, s = allocate s (Evaluated value) in
            (Addr a, s)
      in
      (s, body, bind [ (x, v) ] env)

let transition s =
  match s.control with
  | Eval (e, env) -> (
      let scope = env @ s.globals in
      match e with
      | Nat n -> { s with control = Return (Number n) }
      | App (f, xs) ->
          {
            s with
            control = Enter (lookup scope f);
            arguments = List.map (atom scope) xs @ s.arguments;
          }
      | Con (c, xs) ->
          { s with control = Return (Data (c, List.map (atom scope) xs)) }
      | Prim (op, a, b) ->
          {
            s with
            control = Return (primitive op (atom scope a) (atom scope b));
          }
      | Let (recursive, bindings, body) ->
          let s, env = allocate_bindings s env recursive bindings in
          { s with control = Eval (body, env) }
      | Case (e, alts) ->
          {
            s with
            control = Eval (e, env);
            arguments = [];
            stack = Match (alts, env, s.arguments) :: s.stack;
          })
  | Enter (Int n) -> { s with control = Return (Number n) }
  | Enter (Addr a) -> (
      match lookup_heap s a with
      | Blackhole ->
          failwith ("Black hole: recursive thunk entry at @" ^ string_of_int a)
      | Evaluated value -> { s with control = Return value }
      | Pap (f, xs) ->
          if s.arguments = [] then { s with control = Return (Function a) }
          else { s with control = Enter (Addr f); arguments = xs @ s.arguments }
      | Closure (lf, env) -> (
          match lf.update with
          | Updatable ->
              let s' = set_heap s a Blackhole in
              {
                s' with
                control = Eval (lf.body, env);
                arguments = [];
                stack = Update (a, s.arguments) :: s.stack;
              }
          | Reentrant ->
              let arity = List.length lf.args in
              if List.length s.arguments < arity then
                let a, s =
                  if s.arguments = [] then (a, s)
                  else allocate s (Pap (a, s.arguments))
                in
                { s with control = Return (Function a); arguments = [] }
              else
                let supplied, arguments = take arity s.arguments in
                {
                  s with
                  control =
                    Eval (lf.body, bind (List.combine lf.args supplied) env);
                  arguments;
                }))
  | Return value -> (
      if s.arguments <> [] then
        match value with
        | Function a -> { s with control = Enter (Addr a) }
        | _ -> failwith "Cannot apply a number or constructor"
      else
        match s.stack with
        | Update (a, arguments) :: stack ->
            let s = set_heap s a (Evaluated value) in
            { s with arguments; stack }
        | Match (alts, env, arguments) :: stack ->
            let s, body, env = match_result s value alts env in
            { s with control = Eval (body, env); arguments; stack }
        | [] -> failwith "Evaluation already finished")

let step s () =
  match (s.control, s.arguments, s.stack) with
  | Return value, [], [] -> Ok (Halt value)
  | _ ->
      Utils_base.Error.protect (fun () ->
          Continue { next = transition s; output = () })

let validate_state s =
  Utils_base.Error.protect (fun () ->
      Utils_base.Error.get (Code.validate s.code);
      let globals = List.map fst s.code in
      let expected_globals = List.mapi (fun a x -> (x, Addr a)) globals in
      if s.globals <> expected_globals then
        failwith "Invalid global environment";
      let addresses = List.map fst s.heap |> List.sort Int.compare in
      if
        s.next_addr < 0
        || List.length addresses <> s.next_addr
        || List.mapi (fun i a -> i = a) addresses |> List.exists not
      then failwith "Invalid heap address sequence";
      let address a = ignore (lookup_heap s a) in
      List.iter (function _, Addr a -> address a | _ -> ()) s.globals;
      let value = function Addr a -> address a | Int _ -> () in
      let values = List.iter value in
      let environment env =
        unique "environment binding" (List.map fst env);
        List.iter (fun (_, v) -> value v) env
      in
      let function_address a =
        match lookup_heap s a with
        | Closure ({ update = Reentrant; args = _ :: _; _ }, _) | Pap _ -> ()
        | _ -> failwith "Function must point to a function closure or PAP"
      in
      let result = function
        | Number _ -> ()
        | Data (_, xs) -> values xs
        | Function a -> function_address a
      in
      List.iter
        (fun (_, closure) ->
          match closure with
          | Blackhole -> ()
          | Evaluated v -> result v
          | Pap (f, xs) -> (
              values xs;
              match lookup_heap s f with
              | Closure ({ update = Reentrant; args; _ }, _)
                when xs <> [] && List.length xs < List.length args ->
                  ()
              | _ -> failwith "Invalid partial application")
          | Closure (lf, env) ->
              environment env;
              if List.map fst env <> lf.free then
                failwith "Closure capture mismatch";
              check_lambda globals (List.map fst env @ globals) lf)
        s.heap;
      let updates =
        List.filter_map (function Update (a, _) -> Some a | _ -> None) s.stack
      in
      if List.length (List.sort_uniq Int.compare updates) <> List.length updates
      then failwith "Duplicate update frame";
      List.iter
        (fun (a, c) ->
          if c = Blackhole <> List.mem a updates then
            failwith "Black hole/update frame mismatch")
        s.heap;
      List.iter
        (function
          | Update (a, xs) ->
              address a;
              values xs
          | Match (alts, env, xs) ->
              environment env;
              values xs;
              check_alternatives globals (List.map fst env @ globals) alts)
        s.stack;
      values s.arguments;
      match s.control with
      | Enter v -> value v
      | Return v -> result v
      | Eval (e, env) ->
          environment env;
          check_expr globals (List.map fst env @ globals) e)

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    let* () = validate_state s in
    Ok s
end

let env_text env =
  String.concat ", "
    (List.map (fun (x, v) -> Id.to_string x ^ " = " ^ value_text v) env)

let closure_text = function
  | Closure (lf, env) -> lambda_text lf ^ " {" ^ env_text env ^ "}"
  | Blackhole -> "black hole"
  | Evaluated v -> "updated: " ^ result_text v
  | Pap (a, xs) -> "PAP @" ^ string_of_int a ^ "(" ^ values_text xs ^ ")"

let frame_text = function
  | Update (a, xs) ->
      "update @" ^ string_of_int a ^ "; saved arguments: " ^ values_text xs
  | Match (alts, env, xs) ->
      String.concat "; "
        (List.map (fun (p, e) -> pattern_text p ^ " -> " ^ expr_text e) alts)
      ^ " {" ^ env_text env ^ "}; saved arguments: " ^ values_text xs

let render s =
  let mode, focus, env =
    match s.control with
    | Eval (e, env) -> ("eval", expr_text e, env)
    | Enter v -> ("enter", value_text v, [])
    | Return v -> ("return", result_text v, [])
  in
  let open Utils_runtime.Render in
  [
    text ~title:"code" (Code.print s.code);
    text_table ~title:"control" ~columns:[ "mode"; "focus" ] [ [ mode; focus ] ];
    keyed_table ~title:"environment" ~columns:[ "name"; "value" ]
      ~key_text:Id.to_string value_text env;
    keyed_table ~title:"globals" ~columns:[ "name"; "value" ]
      ~key_text:Id.to_string value_text s.globals;
    indexed_table ~title:"arguments (top first)" value_text s.arguments;
    indexed_table ~title:"return / update stack (top first)" frame_text s.stack;
    keyed_table ~title:"heap" ~columns:[ "address"; "closure" ]
      ~key_text:(fun a -> "@" ^ string_of_int a)
      closure_text
      (List.sort (fun (a, _) (b, _) -> Int.compare a b) s.heap);
  ]
