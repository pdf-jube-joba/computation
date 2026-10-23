open Structured_lang_shared.Arithmetic
open Runtime_types
open Typed_store

let function_lookup code name =
  match List.find_opt (fun (f : fn_decl) -> f.name = name) code.functions with
  | Some f -> f
  | None ->
      failwith ("unknown function: " ^ Utils_names.Identifier.to_string name)

let eval_binop op a b =
  match (op, a, b) with
  | Add, Number_value a, Number_value b -> Number_value (N.add a b)
  | Sub, Number_value a, Number_value b ->
      if N.compare a b < 0 then failwith "operator '-' underflowed";
      Number_value (N.sub a b)
  | Eq, Number_value a, Number_value b -> Bool_value (N.equal a b)
  | Eq, Char_value a, Char_value b -> Bool_value (a = b)
  | Eq, Bool_value a, Bool_value b -> Bool_value (a = b)
  | Eq, Unit_v, Unit_v -> Bool_value true
  | Eq, Ptr_value a, Ptr_value b -> Bool_value (a = b)
  | Eq, FnPtr a, FnPtr b -> Bool_value (a = b)
  | Lt, Number_value a, Number_value b -> Bool_value (N.compare a b < 0)
  | And, Bool_value a, Bool_value b -> Bool_value (a && b)
  | Or, Bool_value a, Bool_value b -> Bool_value (a || b)
  | _ ->
      failwith
        (match op with
        | Add -> "operator '+' expects numbers"
        | Sub -> "operator '-' expects numbers"
        | Eq -> "operator '==' expects primitive, ptr, or fn values"
        | Lt -> "operator '<' expects numbers"
        | And -> "operator '&&' expects bools"
        | Or -> "operator '||' expects bools")

let eval_unop Not = function
  | Bool_value b -> Bool_value (not b)
  | _ -> failwith "operator '!' expects a bool"

let rec eval_place s = function
  | Static n ->
      if not (List.mem_assoc n s.store.statics) then
        failwith ("unknown static: " ^ Utils_names.Identifier.to_string n);
      { key = Static_key n; projections = [] }
  | Local n ->
      let rec find = function
        | [] -> failwith ("unknown local: " ^ Utils_names.Identifier.to_string n)
        | scope :: scopes -> (
            match List.assoc_opt n scope with
            | Some id -> { key = Local_key id; projections = [] }
            | None -> find scopes)
      in
      find s.env
  | Field (p, i) ->
      let loc = eval_place s p in
      { loc with projections = loc.projections @ [ Field_proj i ] }
  | Tag_place (p, i) ->
      let loc = eval_place s p in
      { loc with projections = loc.projections @ [ Tag_proj i ] }
  | Index (p, v) ->
      let i =
        match eval_value s v with
        | Number_value n -> get (N.to_int n)
        | _ -> failwith "array index must be a number"
      in
      let loc = eval_place s p in
      { loc with projections = loc.projections @ [ Index_proj i ] }
  | Deref v -> (
      match eval_value s v with
      | Ptr_value (Some loc) -> loc
      | Ptr_value None -> failwith "cannot dereference \\null_ptr"
      | _ -> failwith "dereference expects a pointer")

and eval_value s = function
  | Number n -> Number_value n
  | Char c -> Char_value c
  | Boolean b -> Bool_value b
  | Unit_value -> Unit_v
  | NullPtr -> Ptr_value None
  | NullFn -> FnPtr None
  | BinOp (a, op, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      eval_binop op a b
  | UnOp (op, v) -> eval_unop op (eval_value s v)
  | Pair vs -> Aggregate (List.map (eval_value s) vs)
  | Tag (t, v) -> Sum_value (t, eval_value s v)
  | Load p -> snd (lookup_location s (eval_place s p))
  | Addr p -> Ptr_value (Some (eval_place s p))
  | Function n ->
      ignore (function_lookup s.code n);
      FnPtr (Some n)
