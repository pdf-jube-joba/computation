module Value_type = struct
  type t =
    | Bit
    | Array of Utils.Number.t * t
    | Strct of (Utils.Identifier.t * t) list
    | Enume of Utils.Identifier.t list
  [@@deriving yojson]
end

module Value = struct
  type t =
    | Bit of bool
    | Array of t list
    | Strct of (Utils.Identifier.t * t) list
    | Enume of Utils.Identifier.t
  [@@deriving yojson]

  let new_bit b = Bit b
  let new_array values = Array values

  (* Preserve the source constructor's inverted HashSet::insert check: its
     first field always returns None. Expression StrctConst bypasses it. *)
  let new_strct = function [] -> Some (Strct []) | _ -> None
  let new_enume name = Enume name

  let get_field_of_value value name =
    match value with Strct fields -> List.assoc_opt name fields | _ -> None

  let rec print = function
    | Bit b -> if b then "#true" else "#false"
    | Array values -> "[" ^ String.concat ", " (List.map print values) ^ "]"
    | Strct fields ->
        "{"
        ^ String.concat ", "
            (List.map (fun (name, value) -> name ^ ": " ^ print value) fields)
        ^ "}"
    | Enume name -> "<" ^ name ^ ">"
end

type value_type = Value_type.t [@@deriving yojson]
type value = Value.t [@@deriving yojson]

let rec typable_value_exn value expected =
  match (value, expected) with
  | Value.Bit _, Value_type.Bit -> ()
  | Value.Array values, Value_type.Array (length, element_type) ->
      if
        not
          (Utils.Number.equal (Utils.Number.of_int (List.length values)) length)
      then failwith "mismatch length of array";
      List.iter (fun value -> typable_value_exn value element_type) values
  | Value.Strct fields, Value_type.Strct field_types ->
      let keys fields = List.map fst fields |> List.sort_uniq String.compare in
      if keys fields <> keys field_types then failwith "mismatch field name";
      List.iter
        (fun key ->
          typable_value_exn (List.assoc key fields) (List.assoc key field_types))
        (keys fields)
  | Value.Enume name, Value_type.Enume names ->
      if not (List.mem name names) then
        failwith
          ("enum " ^ name ^ " is not contained in [" ^ String.concat ", " names
         ^ "]")
  | _ -> failwith "type mismatch"

let typable_value value expected =
  Utils.Json.protect (fun () -> typable_value_exn value expected)

type comb_exp_val =
  | Val of value
  | Var of Utils.Identifier.t
  | And of comb_exp_val * comb_exp_val
  | Or of comb_exp_val * comb_exp_val
  | Not of comb_exp_val
  | IfCond of comb_exp_val * comb_exp_val * comb_exp_val
  | ArrayAcc of comb_exp_val * Utils.Number.t
  | ArrayConst of comb_exp_val list
  | FieldAcc of comb_exp_val * Utils.Identifier.t
  | StrctConst of (Utils.Identifier.t * comb_exp_val) list
  | Switch of comb_exp_val * (Utils.Identifier.t * comb_exp_val) list
  | SeqAndFinal of (Utils.Identifier.t * comb_exp_val) list * comb_exp_val
  | CombModule of Utils.Identifier.t * comb_exp_val
[@@deriving yojson]

type comb_module = {
  name : Utils.Identifier.t;
  input_type : value_type;
  otput_type : value_type;
  comb_func : comb_exp_val;
}
[@@deriving yojson]

type code_env = { code : comb_exp_val; mod_env : comb_module list }
[@@deriving yojson]

let rec eval_exn mod_env var_env expression =
  let eval expression = eval_exn mod_env var_env expression in
  match expression with
  | Val value -> value
  | Var name -> (
      match List.assoc_opt name var_env with
      | Some value -> value
      | None -> failwith ("not found var " ^ name))
  | And (a, b) -> (
      let va = eval a in
      let vb = eval b in
      match (va, vb) with
      | Value.Bit a, Value.Bit b -> Value.Bit (a && b)
      | _ -> failwith "expression is not boolean")
  | Or (a, b) -> (
      let va = eval a in
      let vb = eval b in
      match (va, vb) with
      | Value.Bit a, Value.Bit b -> Value.Bit (a || b)
      | _ -> failwith "expression is not boolean")
  | Not expression -> (
      match eval expression with
      | Value.Bit b -> Value.Bit (not b)
      | _ -> failwith "expression is not boolean")
  | IfCond (condition, consequent, alternative) -> (
      (* Rust evaluates both branches, including the unselected branch. *)
      let condition = eval condition in
      let consequent = eval consequent in
      let alternative = eval alternative in
      match condition with
      | Value.Bit true -> consequent
      | Value.Bit false -> alternative
      | _ -> failwith "expression is not boolean")
  | ArrayAcc (expression, index) -> (
      let index =
        match Utils.Number.to_int index with
        | Ok n -> n
        | Error _ -> failwith "access out of index"
      in
      match eval expression with
      | Value.Array values ->
          if index >= List.length values then failwith "access out of index"
          else List.nth values index
      | _ -> failwith "expression is not array")
  | ArrayConst expressions -> Value.Array (List.map eval expressions)
  | FieldAcc (expression, field) -> (
      match eval expression with
      | Value.Strct fields -> (
          match List.assoc_opt field fields with
          | Some value -> value
          | None -> failwith ("not found field " ^ field))
      | _ -> failwith "expression is not struct")
  | StrctConst fields ->
      Value.Strct
        (List.map (fun (name, expression) -> (name, eval expression)) fields)
  | Switch (expression, cases) -> (
      let value = eval expression in
      let cases =
        List.map (fun (name, expression) -> (name, eval expression)) cases
      in
      match value with
      | Value.Enume name -> (
          match List.assoc_opt name cases with
          | Some value -> value
          | None -> failwith "switch miss")
      | _ -> failwith "expression is not enume")
  | SeqAndFinal (sequence, final_value) ->
      let vars =
        List.fold_left
          (fun vars (name, expression) ->
            let value = eval_exn mod_env vars expression in
            vars @ [ (name, value) ])
          var_env sequence
      in
      eval_exn mod_env vars final_value
  | CombModule (name, expression) ->
      let comb_module =
        match List.find_opt (fun m -> m.name = name) mod_env with
        | Some m -> m
        | None -> failwith ("module name " ^ name ^ " is not found")
      in
      let value = eval expression in
      (* The implemented evaluator does not check declared module types. *)
      eval_exn mod_env [ ("IN", value) ] comb_module.comb_func

let eval_to_val mod_env var_env expression =
  Utils.Json.protect (fun () -> eval_exn mod_env var_env expression)

type simple_module = {
  state_type_and_init : value * value_type;
  input_type : value_type;
  otput_type : value_type;
  trans_func : comb_exp_val;
  otput_func : comb_exp_val;
}
[@@deriving yojson]

type simple_module_state = {
  state : value;
  trans_func : code_env;
  otput_func : code_env;
}
[@@deriving yojson]

type general_module =
  | Simple of simple_module
  | Graph of graph_module
  | Iter of iter_module

and graph_module = {
  state_name_machines : (Utils.Identifier.t * general_module) list;
  input_type : value_type;
  otput_type : value_type;
  trans_func : comb_exp_val;
  otput_func : comb_exp_val;
}

and iter_module = {
  state_machine : general_module;
  trans_func : comb_exp_val;
  input_func : comb_exp_val;
  otput_func : comb_exp_val;
}
[@@deriving yojson]

type general_module_state =
  | SimpleState of simple_module_state
  | GraphState of graph_module_state
  | IterState of iter_module_state

and graph_module_state = {
  states_machines : (Utils.Identifier.t * general_module_state) list;
  trans_func : code_env;
  otput_func : code_env;
}

and iter_module_state = {
  init_state_machine : general_module_state;
  state_machines : general_module_state list;
  trans_func : code_env;
  input_func : code_env;
  otput_func : code_env;
}
[@@deriving yojson]

let field_in = "IN"
let field_out = "OUT"
let field_state = "STATE"
let field_next = "NEXT"
let field_this = "THIS"
let field_prev = "PREV"
let simple_now_state (machine : simple_module_state) = machine.state

let simple_get_otput (machine : simple_module_state) =
  eval_to_val machine.otput_func.mod_env
    [ (field_state, machine.state) ]
    machine.otput_func.code

let simple_clock (machine : simple_module_state) input =
  Result.map
    (fun state -> { machine with state })
    (eval_to_val machine.trans_func.mod_env
       [ (field_in, input); (field_state, machine.state) ]
       machine.trans_func.code)

let unsupported feature =
  Error (feature ^ " was not implemented in the original Rust HDL")

let build_simple (_ : simple_module) : (simple_module_state, string) result =
  unsupported "Simple module construction"

let general_now_state (_ : general_module_state) : (value, string) result =
  unsupported "General module state dispatch"

let general_get_otput (_ : general_module_state) : (value, string) result =
  unsupported "General module output dispatch"

let general_clock (_ : general_module_state) (_ : value) :
    (general_module_state, string) result =
  unsupported "General module clock dispatch"

let result_exn = function
  | Ok value -> value
  | Error message -> failwith message

let graph_now_state (machine : graph_module_state) =
  Utils.Json.protect (fun () ->
      Value.Strct
        (List.map
           (fun (name, state) -> (name, result_exn (general_now_state state)))
           machine.states_machines))

let graph_get_otput (machine : graph_module_state) =
  Utils.Json.protect (fun () ->
      let vars =
        List.map
          (fun (name, state) -> (name, result_exn (general_get_otput state)))
          machine.states_machines
      in
      (* The original implementation evaluates trans_func here, not otput_func. *)
      eval_exn machine.trans_func.mod_env vars machine.trans_func.code)

let graph_clock (machine : graph_module_state) input =
  Utils.Json.protect (fun () ->
      let vars =
        (field_in, input)
        :: List.map
             (fun (name, state) -> (name, result_exn (general_get_otput state)))
             machine.states_machines
      in
      let value =
        eval_exn machine.trans_func.mod_env vars machine.trans_func.code
      in
      let states_machines =
        List.map
          (fun (name, state) ->
            let next_input =
              match Value.get_field_of_value value name with
              | Some v -> v
              | None -> failwith ("field " ^ name ^ " not found")
            in
            (name, result_exn (general_clock state next_input)))
          machine.states_machines
      in
      { machine with states_machines })

let iter_get_otput (machine : iter_module_state) =
  Utils.Json.protect (fun () ->
      let first =
        match machine.state_machines with
        | state :: _ -> state
        | [] -> failwith "Iterator has no first module state"
      in
      let output = result_exn (general_get_otput first) in
      eval_exn machine.otput_func.mod_env
        [ (field_state, output) ]
        machine.otput_func.code)

let iter_clock (machine : iter_module_state) input =
  Utils.Json.protect (fun () ->
      let previous =
        eval_exn machine.input_func.mod_env
          [ (field_in, input) ]
          machine.input_func.code
      in
      let outputs =
        List.map
          (fun state -> result_exn (general_get_otput state))
          machine.state_machines
      in
      let other = result_exn (general_get_otput machine.init_state_machine) in
      let count = List.length outputs in
      let at n =
        if n < 0 || n >= count then
          failwith "HDL iterator neighbor index is out of bounds"
        else List.nth outputs n
      in
      let state_machines =
        List.mapi
          (fun n state ->
            let prev =
              if n = 0 then previous
              else if n - 1 > count then other
              else at (n - 1)
            in
            let current = if n > count then other else at n in
            let next = if n + 1 > count then other else at (n + 1) in
            (* Preserve the source's repeated PREV binding. General dispatch remains
       unsupported, so these dormant operations cannot be entered today. *)
            let input =
              eval_exn machine.trans_func.mod_env
                [
                  (field_prev, prev); (field_this, current); (field_prev, next);
                ]
                machine.trans_func.code
            in
            result_exn (general_clock state input))
          machine.state_machines
      in
      { machine with state_machines })

module Parser = struct
  module L = Utils.Lexer

  let name = L.identifier
  let block p closing item = L.many_until ~skip_separator:',' p ~closing item

  let rec value p =
    if L.eat p "#true" then Value.Bit true
    else if L.eat p "#false" then Value.Bit false
    else if L.eat_symbol p '[' then Value.Array (block p ']' value)
    else if L.eat_symbol p '{' then
      let fields =
        block p '}' (fun p ->
            let name = name p in
            L.expect_symbol p ':';
            let value = value p in
            (name, value))
      in
      match Value.new_strct fields with
      | Some value -> value
      | None -> L.error p "field 名がかぶってそう"
    else if L.eat_symbol p '<' then (
      let name = name p in
      L.expect_symbol p '>';
      Value.Enume name)
    else L.error p "expected value"

  let parse_value = L.parse ~nested_comments:false value
end

module Interface = struct
  let parse_value = Parser.parse_value
end

let parse_value = Parser.parse_value

(* No Machine.S or compiler api is exposed: Rust had no complete source-code
   parser, machine constructor, general module interpreter, or compiler. *)
let unsupported_capabilities =
  [
    "Type/expression/module source parser";
    "Simple module construction";
    "General module state/output/clock dispatch";
    "HDL-to-logic-circuit compiler";
  ]
