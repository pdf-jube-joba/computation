include Syntax
module Program = Program

type code_env = Program.t [@@deriving yojson]

let eval_to_val mod_env var_env expression =
  let open Utils_base.Error in
  let inputs =
    List.sort_uniq Utils_names.Identifier.compare (List.map fst var_env)
  in
  let* program = Program.make ~modules:mod_env ~inputs expression in
  Program.eval program var_env

let eval_exn mod_env var_env expression =
  Utils_base.Error.get (eval_to_val mod_env var_env expression)

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
  state_name_machines : (Utils_names.Identifier.t * general_module) list;
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
  states_machines : (Utils_names.Identifier.t * general_module_state) list;
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

let field_in = Utils_names.Identifier.make_exn "IN"
let field_out = Utils_names.Identifier.make_exn "OUT"
let field_state = Utils_names.Identifier.make_exn "STATE"
let field_next = Utils_names.Identifier.make_exn "NEXT"
let field_this = Utils_names.Identifier.make_exn "THIS"
let field_prev = Utils_names.Identifier.make_exn "PREV"

let simple_get_otput (machine : simple_module_state) =
  Program.eval machine.otput_func [ (field_state, machine.state) ]

let simple_clock (machine : simple_module_state) input =
  Result.map
    (fun state -> { machine with state })
    (Program.eval machine.trans_func
       [ (field_in, input); (field_state, machine.state) ])

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
  Utils_base.Error.protect (fun () ->
      Value.Strct
        (List.map
           (fun (name, state) -> (name, result_exn (general_now_state state)))
           machine.states_machines))

let graph_get_otput (machine : graph_module_state) =
  Utils_base.Error.protect (fun () ->
      let vars =
        List.map
          (fun (name, state) -> (name, result_exn (general_get_otput state)))
          machine.states_machines
      in
      (* The original implementation evaluates trans_func here, not otput_func. *)
      result_exn (Program.eval machine.trans_func vars))

let graph_clock (machine : graph_module_state) input =
  Utils_base.Error.protect (fun () ->
      let vars =
        (field_in, input)
        :: List.map
             (fun (name, state) -> (name, result_exn (general_get_otput state)))
             machine.states_machines
      in
      let value = result_exn (Program.eval machine.trans_func vars) in
      let states_machines =
        List.map
          (fun (name, state) ->
            let next_input =
              match Value.get_field_of_value value name with
              | Some v -> v
              | None ->
                  failwith
                    ("field "
                    ^ Utils_names.Identifier.to_string name
                    ^ " not found")
            in
            (name, result_exn (general_clock state next_input)))
          machine.states_machines
      in
      { machine with states_machines })

let iter_get_otput (machine : iter_module_state) =
  Utils_base.Error.protect (fun () ->
      let first =
        match machine.state_machines with
        | state :: _ -> state
        | [] -> failwith "Iterator has no first module state"
      in
      let output = result_exn (general_get_otput first) in
      result_exn (Program.eval machine.otput_func [ (field_state, output) ]))

let iter_clock (machine : iter_module_state) input =
  Utils_base.Error.protect (fun () ->
      let previous =
        result_exn (Program.eval machine.input_func [ (field_in, input) ])
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
              result_exn
                (Program.eval machine.trans_func
                   [
                     (field_prev, prev);
                     (field_this, current);
                     (field_prev, next);
                   ])
            in
            result_exn (general_clock state input))
          machine.state_machines
      in
      { machine with state_machines })

module Parser = struct
  module L = Utils_parsing.Parser

  let block p delimiter item =
    L.group p delimiter (fun p -> L.many ~skip_separator:',' p item)

  let rec value p =
    if L.bump_if p "\\true" then Value.Bit true
    else if L.bump_if p "\\false" then Value.Bit false
    else if L.is_group p L.Brackets then Value.Array (block p L.Brackets value)
    else if L.is_group p L.Braces then
      let fields =
        block p L.Braces (fun p ->
            let name = L.identifier p in
            L.expect_symbol p ':';
            let value = value p in
            (name, value))
      in
      match Value.new_strct fields with
      | Some value -> value
      | None -> L.error p "field 名がかぶってそう"
    else if L.bump_if_symbol p '<' then (
      let name = L.identifier p in
      L.expect_symbol p '>';
      Value.Enume name)
    else L.error p "expected value"

  let parse_value = L.parse ~nested_comments:false value
end
