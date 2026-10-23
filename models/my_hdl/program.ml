open Syntax
module Identifier = Utils_names.Identifier
module Var_id = Utils_names.Id.Make ()
module Module_id = Utils_names.Id.Make ()
module Modules = Utils_names.Intern.Make (Identifier) (Module_id)

type expression = (Var_id.t, Module_id.t) expr

type resolved = {
  code : expression;
  inputs : (Identifier.t * Var_id.t) list;
  dependencies : Module_id.Set.t;
}

type source = {
  code : comb_exp_val;
  mod_env : comb_module list;
  inputs : Identifier.t list;
}
[@@deriving yojson]

type t = { source : source; modules : resolved array; root : resolved }

let input_name = Identifier.make_exn "IN"

let resolve modules inputs code =
  let supply = ref Var_id.Supply.empty in
  let dependencies = ref Module_id.Set.empty in
  let lookup kind name names =
    match Identifier.Map.find_opt name names with
    | Some id -> id
    | None -> failwith ("Unknown " ^ kind ^ ": " ^ Identifier.to_string name)
  in
  let inputs, scope =
    List.fold_left
      (fun (inputs, scope) name ->
        if Identifier.Map.mem name scope then
          failwith ("Duplicate input: " ^ Identifier.to_string name);
        let id = Var_id.Supply.take supply in
        ((name, id) :: inputs, Identifier.Map.add name id scope))
      ([], Identifier.Map.empty) inputs
  in
  let rec visit scope = function
    | Val value -> Val value
    | Var name -> Var (lookup "variable" name scope)
    | And (a, b) -> And (visit scope a, visit scope b)
    | Or (a, b) -> Or (visit scope a, visit scope b)
    | Not expression -> Not (visit scope expression)
    | IfCond (condition, consequent, alternative) ->
        IfCond
          ( visit scope condition,
            visit scope consequent,
            visit scope alternative )
    | ArrayAcc (expression, index) -> ArrayAcc (visit scope expression, index)
    | ArrayConst expressions -> ArrayConst (List.map (visit scope) expressions)
    | FieldAcc (expression, field) -> FieldAcc (visit scope expression, field)
    | StrctConst fields ->
        StrctConst (List.map (fun (name, e) -> (name, visit scope e)) fields)
    | Switch (expression, cases) ->
        Switch
          ( visit scope expression,
            List.map (fun (name, e) -> (name, visit scope e)) cases )
    | SeqAndFinal (sequence, final_value) ->
        let sequence, scope =
          List.fold_left
            (fun (sequence, scope) (name, expression) ->
              let expression = visit scope expression in
              let id = Var_id.Supply.take supply in
              let scope =
                if Identifier.Map.mem name scope then scope
                else Identifier.Map.add name id scope
              in
              ((id, expression) :: sequence, scope))
            ([], scope) sequence
        in
        SeqAndFinal (List.rev sequence, visit scope final_value)
    | CombModule (name, expression) ->
        let id =
          match Modules.find name modules with
          | Some id -> id
          | None -> failwith ("Unknown module: " ^ Identifier.to_string name)
        in
        dependencies := Module_id.Set.add id !dependencies;
        CombModule (id, visit scope expression)
  in
  let code = visit scope code in
  { code; inputs = List.rev inputs; dependencies = !dependencies }

let check_cycles declarations modules =
  let marks = Array.make (Array.length modules) 0 in
  let rec visit path id =
    let index = Module_id.to_int id in
    match marks.(index) with
    | 2 -> ()
    | 1 ->
        let names =
          List.rev (id :: path)
          |> List.map (fun id ->
              Identifier.to_string declarations.(Module_id.to_int id).name)
        in
        failwith ("Combinational module cycle: " ^ String.concat " -> " names)
    | _ ->
        marks.(index) <- 1;
        Module_id.Set.iter (visit (id :: path)) modules.(index).dependencies;
        marks.(index) <- 2
  in
  Array.iter (fun m -> Module_id.Set.iter (visit []) m.dependencies) modules

let make ~modules ~inputs code =
  Utils_base.Error.protect (fun () ->
      let declarations = Array.of_list modules in
      let names = ref Modules.empty in
      Array.iter
        (fun (m : comb_module) ->
          if Modules.find m.name !names <> None then
            failwith ("Duplicate module: " ^ Identifier.to_string m.name);
          ignore (Modules.take m.name names))
        declarations;
      let modules =
        Array.map
          (fun (m : comb_module) ->
            try resolve !names [ input_name ] m.comb_func
            with Failure message ->
              failwith ("Module " ^ Identifier.to_string m.name ^ ": " ^ message))
          declarations
      in
      check_cycles declarations modules;
      let root = resolve !names inputs code in
      {
        source = { code; mod_env = Array.to_list declarations; inputs };
        modules;
        root;
      })

let eval program inputs =
  Utils_base.Error.protect (fun () ->
      let rec evaluate vars = function
        | Val value -> value
        | Var id -> Var_id.Map.find id vars
        | And (a, b) -> (
            let a = evaluate vars a in
            let b = evaluate vars b in
            match (a, b) with
            | Value.Bit a, Value.Bit b -> Value.Bit (a && b)
            | _ -> failwith "expression is not boolean")
        | Or (a, b) -> (
            let a = evaluate vars a in
            let b = evaluate vars b in
            match (a, b) with
            | Value.Bit a, Value.Bit b -> Value.Bit (a || b)
            | _ -> failwith "expression is not boolean")
        | Not expression -> (
            match evaluate vars expression with
            | Value.Bit b -> Value.Bit (not b)
            | _ -> failwith "expression is not boolean")
        | IfCond (condition, consequent, alternative) -> (
            let condition = evaluate vars condition in
            let consequent = evaluate vars consequent in
            let alternative = evaluate vars alternative in
            match condition with
            | Value.Bit true -> consequent
            | Value.Bit false -> alternative
            | _ -> failwith "expression is not boolean")
        | ArrayAcc (expression, index) -> (
            let index =
              match Utils_base.Number.to_int index with
              | Ok n -> n
              | Error _ -> failwith "access out of index"
            in
            match evaluate vars expression with
            | Value.Array values ->
                if index >= List.length values then
                  failwith "access out of index"
                else List.nth values index
            | _ -> failwith "expression is not array")
        | ArrayConst expressions ->
            Value.Array (List.map (evaluate vars) expressions)
        | FieldAcc (expression, field) -> (
            match evaluate vars expression with
            | Value.Strct fields -> (
                match List.assoc_opt field fields with
                | Some value -> value
                | None ->
                    failwith ("not found field " ^ Identifier.to_string field))
            | _ -> failwith "expression is not struct")
        | StrctConst fields ->
            Value.Strct
              (List.map (fun (name, e) -> (name, evaluate vars e)) fields)
        | Switch (expression, cases) -> (
            let value = evaluate vars expression in
            let cases =
              List.map (fun (name, e) -> (name, evaluate vars e)) cases
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
                (fun vars (id, expression) ->
                  let value = evaluate vars expression in
                  Var_id.Map.add id value vars)
                vars sequence
            in
            evaluate vars final_value
        | CombModule (id, expression) ->
            let m = program.modules.(Module_id.to_int id) in
            let value = evaluate vars expression in
            let input_id = snd (List.hd m.inputs) in
            evaluate (Var_id.Map.singleton input_id value) m.code
      in
      let vars =
        List.fold_left
          (fun vars (name, id) ->
            let value =
              match List.assoc_opt name inputs with
              | Some value -> value
              | None -> failwith ("Missing input: " ^ Identifier.to_string name)
            in
            Var_id.Map.add id value vars)
          Var_id.Map.empty program.root.inputs
      in
      evaluate vars program.root.code)

let to_yojson program = source_to_yojson program.source

let of_yojson json =
  let open Utils_base.Error in
  let* source = source_of_yojson json in
  make ~modules:source.mod_env ~inputs:source.inputs source.code
