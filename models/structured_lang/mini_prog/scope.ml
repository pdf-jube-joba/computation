open Runtime_types
open Typed_store
open Eval

let allocate_scope s (bindings : (Utils_names.Identifier.t * ty) list) values =
  if List.length bindings <> List.length values then
    failwith "binding count mismatch";
  let next = ref s.next_local_id in
  let env, cells =
    List.fold_left
      (fun (env, cells) ((name, ty), value) ->
        if not (type_match value ty) then
          failwith ("type mismatch for " ^ Utils_names.Identifier.to_string name);
        let id = Local_id.Supply.take next in
        ( Utils_base.Assoc.set name id env,
          Utils_base.Assoc.set id { ty; value } cells ))
      ([], [])
      (List.combine bindings values)
  in
  (env, cells, !next)

let enter_function s name args =
  let f = function_lookup s.code name in
  if List.length f.block.bindings <> List.length args then
    failwith
      (Printf.sprintf "argument count mismatch for %s: expected %d, got %d"
         (Utils_names.Identifier.to_string name)
         (List.length f.block.bindings)
         (List.length args));
  List.iter2
    (fun (var, ty) value ->
      if not (type_match value ty) then
        failwith
          ("argument type mismatch for " ^ Utils_names.Identifier.to_string var))
    f.block.bindings args;
  let env, cells, next_local_id = allocate_scope s f.block.bindings args in
  {
    s with
    current_function = f.name;
    env = [ env ];
    store = { s.store with locals = [ cells ] :: s.store.locals };
    inner_k = List.map (fun s -> Stmt_frame s) f.block.stmts;
    next_local_id;
  }

let push_scope s bindings values =
  match s.store.locals with
  | [] -> failwith "push-scope without current function frame"
  | scopes :: fs ->
      let env, cells, next_local_id = allocate_scope s bindings values in
      {
        s with
        env = env :: s.env;
        store = { s.store with locals = (cells :: scopes) :: fs };
        next_local_id;
      }

let pop_scope s =
  match (s.env, s.store.locals) with
  | [], _ -> failwith "scope underflow in env"
  | _, [] -> failwith "scope underflow in locals"
  | _, [] :: _ -> failwith "scope underflow in current function"
  | _ :: env, (_ :: scopes) :: fs ->
      { s with env; store = { s.store with locals = scopes :: fs } }
