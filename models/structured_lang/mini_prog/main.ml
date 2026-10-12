open Structured_lang_shared.Arithmetic
include Syntax
module Code = Parser.Code

let rec type_match v ty =
  match (v, ty) with
  | Number_value _, Num -> true
  | Number_value n, U8 -> N.compare n "255" <= 0
  | Bool_value _, Bool | Unit_v, Unit | Ptr_value _, Ptr | FnPtr _, Fn -> true
  | Aggregate vs, Product ts ->
      List.length vs = List.length ts && List.for_all2 type_match vs ts
  | Aggregate vs, Array (t, n) ->
      List.length vs = n && List.for_all (fun v -> type_match v t) vs
  | Sum_value (tag, v), Sum ts -> (
      match List.nth_opt ts tag with Some t -> type_match v t | None -> false)
  | _ -> false

let rec default_value = function
  | Num | U8 -> Number_value N.zero
  | Bool -> Bool_value false
  | Unit -> Unit_v
  | Ptr -> Ptr_value None
  | Fn -> FnPtr None
  | Product ts -> Aggregate (List.map default_value ts)
  | Array (t, n) -> Aggregate (List.init n (fun _ -> default_value t))
  | Sum [] -> failwith "sum type must have at least one variant"
  | Sum (t :: _) -> Sum_value (0, default_value t)

let rec project_cell ty value = function
  | [] -> (ty, value)
  | Field_proj i :: ps -> (
      match (ty, value) with
      | Product ts, Aggregate vs ->
          project_cell
            (Utils.List_ext.nth ts i "projection out of bounds")
            (Utils.List_ext.nth vs i "projection out of bounds")
            ps
      | _ -> failwith "invalid place projection")
  | Index_proj i :: ps -> (
      match (ty, value) with
      | Array (t, n), Aggregate vs ->
          if i >= n then failwith "projection out of bounds";
          project_cell t (Utils.List_ext.nth vs i "projection out of bounds") ps
      | _ -> failwith "invalid place projection")
  | Tag_proj i :: ps -> (
      match (ty, value) with
      | Sum ts, Sum_value (tag, v) ->
          if i <> tag then failwith "sum tag mismatch";
          project_cell (Utils.List_ext.nth ts i "sum tag out of bounds") v ps
      | _ -> failwith "invalid place projection")

let rec update_projection ty current ps value =
  match ps with
  | [] ->
      if type_match value ty then value
      else failwith "assigned value does not match location type"
  | Field_proj i :: ps -> (
      match (ty, current) with
      | Product ts, Aggregate vs ->
          let t = Utils.List_ext.nth ts i "field projection out of bounds"
          and old = Utils.List_ext.nth vs i "field projection out of bounds" in
          Aggregate
            (Utils.List_ext.replace vs i
               (update_projection t old ps value)
               "field projection out of bounds")
      | _ -> failwith "invalid assignment target")
  | Index_proj i :: ps -> (
      match (ty, current) with
      | Array (t, n), Aggregate vs ->
          if i >= n then failwith "array index out of bounds";
          let old = Utils.List_ext.nth vs i "array index out of bounds" in
          Aggregate
            (Utils.List_ext.replace vs i
               (update_projection t old ps value)
               "array index out of bounds")
      | _ -> failwith "invalid assignment target")
  | Tag_proj i :: ps -> (
      match (ty, current) with
      | Sum ts, Sum_value (tag, v) ->
          if i <> tag then failwith "sum tag mismatch";
          let t = Utils.List_ext.nth ts i "sum tag out of bounds" in
          Sum_value (tag, update_projection t v ps value)
      | _ -> failwith "invalid assignment target")

let get_cell s = function
  | Heap id ->
      Utils.Assoc.lookup id s.store.heap
        ("unknown heap location: " ^ string_of_int (Heap_id.to_int id))
  | Static_key name ->
      Utils.Assoc.lookup name s.store.statics
        ("unknown static location: " ^ name)
  | Local_key id ->
      let rec functions = function
        | [] ->
            failwith
              ("unknown local location: " ^ string_of_int (Local_id.to_int id))
        | scopes :: fs ->
            let rec find = function
              | [] -> functions fs
              | cells :: scopes -> (
                  match List.assoc_opt id cells with
                  | Some c -> c
                  | None -> find scopes)
            in
            find scopes
      in
      functions s.store.locals

let set_cell s key cell =
  ignore (get_cell s key);
  let store =
    match key with
    | Heap id -> { s.store with heap = Utils.Assoc.set id cell s.store.heap }
    | Static_key name ->
        { s.store with statics = Utils.Assoc.set name cell s.store.statics }
    | Local_key id ->
        let rec funcs = function
          | [] -> assert false
          | scopes :: fs -> (
              let rec scope_loop = function
                | [] -> None
                | cells :: scopes ->
                    if List.mem_assoc id cells then
                      Some (Utils.Assoc.set id cell cells :: scopes)
                    else
                      Option.map
                        (fun scopes -> cells :: scopes)
                        (scope_loop scopes)
              in
              match scope_loop scopes with
              | Some scopes -> scopes :: fs
              | None -> scopes :: funcs fs)
        in
        { s.store with locals = funcs s.store.locals }
  in
  { s with store }

let lookup_location s loc =
  let cell = get_cell s loc.key in
  project_cell cell.ty cell.value loc.projections

let store_assign s loc value =
  let cell = get_cell s loc.key in
  set_cell s loc.key
    {
      cell with
      value = update_projection cell.ty cell.value loc.projections value;
    }

let function_lookup code name =
  match List.find_opt (fun (f : fn_decl) -> f.name = name) code.functions with
  | Some f -> f
  | None -> failwith ("unknown function: " ^ name)

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
        failwith ("unknown static: " ^ n);
      { key = Static_key n; projections = [] }
  | Local n ->
      let rec find = function
        | [] -> failwith ("unknown local: " ^ n)
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
      | Ptr_value None -> failwith "cannot dereference #null_ptr"
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

let allocate_scope s bindings values =
  if List.length bindings <> List.length values then
    failwith "binding count mismatch";
  let next = ref s.next_local_id in
  let env, cells =
    List.fold_left
      (fun (env, cells) ((name, ty), value) ->
        if not (type_match value ty) then failwith ("type mismatch for " ^ name);
        let id = Local_id.Supply.take next in
        (Utils.Assoc.set name id env, Utils.Assoc.set id { ty; value } cells))
      ([], [])
      (List.combine bindings values)
  in
  (env, cells, !next)

let enter_function s name args =
  let f = function_lookup s.code name in
  if List.length f.block.bindings <> List.length args then
    failwith
      (Printf.sprintf "argument count mismatch for %s: expected %d, got %d" name
         (List.length f.block.bindings)
         (List.length args));
  List.iter2
    (fun (var, ty) value ->
      if not (type_match value ty) then
        failwith ("argument type mismatch for " ^ var))
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

let validate (code : code) =
  Utils.Identifier.check_unique
    ~message:(fun n -> "duplicate function: " ^ n)
    (List.map (fun (f : fn_decl) -> f.name) code.functions)

let make code input =
  L.protect (fun () ->
      validate code;
      let statics =
        List.fold_left
          (fun ss (d : static_decl) ->
            Utils.Assoc.set d.name { ty = d.ty; value = default_value d.ty } ss)
          [] code.statics
      in
      let s =
        {
          code;
          current_function = "main";
          exec = Ctrl Next;
          env = [];
          store = { statics; heap = []; locals = [] };
          inner_k = [];
          fn_k = [];
          next_local_id = Local_id.Supply.empty;
          next_heap_id = Heap_id.Supply.empty;
        }
      in
      enter_function s "main" (List.map (fun n -> Number_value n) input))

let step_eval s = function
  | Assign (p, v) ->
      let loc = eval_place s p in
      let v = eval_value s v in
      { (store_assign s loc v) with exec = Ctrl Next }
  | If (c, stmt) -> (
      match eval_value s c with
      | Bool_value true -> { s with exec = Eval stmt }
      | Bool_value false -> { s with exec = Ctrl Next }
      | _ -> failwith "if expects a bool")
  | Case (tag, v, stmt) -> (
      match eval_value s v with
      | Sum_value (found, _) ->
          { s with exec = (if tag = found then Eval stmt else Ctrl Next) }
      | _ -> failwith "case expects a tagged sum")
  | HAlloc (ty, p) ->
      let id, next_heap_id = Heap_id.Supply.fresh s.next_heap_id in
      let s =
        {
          s with
          next_heap_id;
          store =
            {
              s.store with
              heap =
                Utils.Assoc.set id { ty; value = default_value ty } s.store.heap;
            };
        }
      in
      let loc = eval_place s p in
      {
        (store_assign s loc
           (Ptr_value (Some { key = Heap id; projections = [] })))
        with
        exec = Ctrl Next;
      }
  | HFree v -> (
      let loc =
        match eval_value s v with
        | Ptr_value (Some loc) -> loc
        | _ -> failwith "hfree expects a heap pointer"
      in
      match (loc.key, loc.projections) with
      | Heap id, [] ->
          if not (List.mem_assoc id s.store.heap) then
            failwith "hfree on unknown heap location";
          {
            s with
            store = { s.store with heap = List.remove_assoc id s.store.heap };
            exec = Ctrl Next;
          }
      | _ -> failwith "hfree expects a top-level heap pointer")
  | Loop (label, stmt) ->
      {
        s with
        inner_k = Loop_frame (label, stmt) :: s.inner_k;
        exec = Ctrl Next;
      }
  | Break label -> { s with exec = Ctrl (Break_ctrl label) }
  | Continue label -> { s with exec = Ctrl (Continue_ctrl label) }
  | Block b ->
      let values = List.map (fun (_, ty) -> default_value ty) b.bindings in
      let s = push_scope s b.bindings values in
      {
        s with
        inner_k = List.map (fun s -> Stmt_frame s) b.stmts @ (Scope :: s.inner_k);
        exec = Ctrl Next;
      }
  | Call (callee, args, rets) ->
      let ret_locs = List.map (eval_place s) rets in
      let name =
        match eval_value s callee with
        | FnPtr (Some n) -> n
        | _ -> failwith "call expects a function pointer"
      in
      let values = List.map (eval_value s) args in
      let s =
        {
          s with
          fn_k =
            {
              function_name = s.current_function;
              env = s.env;
              ret_locs;
              inner_k = s.inner_k;
            }
            :: s.fn_k;
        }
      in
      { (enter_function s name values) with exec = Ctrl Next }
  | Return vs -> { s with exec = Ret (List.map (eval_value s) vs) }

let step_ctrl s control =
  match s.inner_k with
  | [] -> failwith "control state has no continuation"
  | frame :: inner_k -> (
      let s = { s with inner_k } in
      match frame with
      | Scope -> { (pop_scope s) with exec = Ctrl control }
      | Stmt_frame stmt ->
          {
            s with
            exec = (match control with Next -> Eval stmt | _ -> Ctrl control);
          }
      | Loop_frame (label, stmt) -> (
          match control with
          | Next -> { s with inner_k = frame :: s.inner_k; exec = Eval stmt }
          | Break_ctrl target ->
              {
                s with
                exec = Ctrl (if target = label then Next else Break_ctrl target);
              }
          | Continue_ctrl target ->
              if target = label then
                { s with inner_k = frame :: s.inner_k; exec = Ctrl Next }
              else { s with exec = Ctrl (Break_ctrl target) }))

let step_ret s values =
  match s.fn_k with
  | [] -> failwith "return without caller"
  | frame :: fn_k ->
      if List.length values <> List.length frame.ret_locs then
        failwith
          (Printf.sprintf "return count mismatch: expected %d, got %d"
             (List.length frame.ret_locs)
             (List.length values));
      let locals =
        match s.store.locals with
        | _ :: ls -> ls
        | [] -> failwith "local store underflow on return"
      in
      let s = { s with fn_k; store = { s.store with locals } } in
      let s =
        List.fold_left
          (fun s (loc, v) -> store_assign s loc v)
          s
          (List.combine frame.ret_locs values)
      in
      {
        s with
        current_function = frame.function_name;
        env = frame.env;
        inner_k = frame.inner_k;
        exec = Ctrl Next;
      }

let step s () =
  L.protect (fun () ->
      let continue next = Utils.Machine.Continue { next; output = "" } in
      match s.exec with
      | Eval stmt -> continue (step_eval s stmt)
      | Ctrl control ->
          if s.fn_k = [] && s.inner_k = [] && control = Next && s.env = [] then
            failwith "main finished without return";
          continue (step_ctrl s control)
      | Ret values ->
          if s.fn_k = [] then
            match values with
            | [ Number_value n ] -> Utils.Machine.Halt n
            | [ _ ] -> failwith "main must return a natural number"
            | _ -> failwith "main must return exactly one value"
          else continue (step_ret s values))

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      L.protect (fun () ->
          validate s.code;
          let locals = ref Local_id.Set.empty
          and heaps = ref Heap_id.Set.empty in
          let key = function
            | Local_key id -> locals := Local_id.Set.add id !locals
            | Heap id -> heaps := Heap_id.Set.add id !heaps
            | Static_key _ -> ()
          in
          let rec value = function
            | Ptr_value (Some loc) -> key loc.key
            | Aggregate values -> List.iter value values
            | Sum_value (_, v) -> value v
            | _ -> ()
          in
          let env entries =
            List.iter (List.iter (fun (_, id) -> key (Local_key id))) entries
          in
          env s.env;
          List.iter
            (fun (frame : fn_frame) ->
              env frame.env;
              List.iter (fun loc -> key loc.key) frame.ret_locs)
            s.fn_k;
          List.iter
            (fun (id, c) ->
              key (Heap id);
              value c.value)
            s.store.heap;
          List.iter (fun (_, c) -> value c.value) s.store.statics;
          List.iter
            (List.iter
               (List.iter (fun (id, c) ->
                    key (Local_key id);
                    value c.value)))
            s.store.locals;
          get (Local_id.Supply.validate_used !locals s.next_local_id);
          get (Heap_id.Supply.validate_used !heaps s.next_heap_id);
          s))

let render s =
  let open Utils.Render in
  let cell_rows to_int cells =
    List.map
      (fun (id, c) ->
        [ string_of_int (to_int id); Code.type_text c.ty; value_text c.value ])
      (Utils.Assoc.sorted cells)
  in
  state
    [
      text_table ~title:"status" ~columns:[ "field"; "value" ]
        [
          [ "function"; s.current_function ];
          [ "exec"; exec_text s.exec ];
          [
            "next_local_id";
            string_of_int (Local_id.Supply.to_int s.next_local_id);
          ];
          [
            "next_heap_id"; string_of_int (Heap_id.Supply.to_int s.next_heap_id);
          ];
        ];
      text_table ~title:"env"
        ~columns:[ "scope"; "name"; "local" ]
        (List.concat
           (List.mapi
              (fun i scope ->
                List.map
                  (fun (n, id) ->
                    [ string_of_int i; n; string_of_int (Local_id.to_int id) ])
                  (Utils.Assoc.sorted scope))
              s.env));
      text_table ~title:"statics"
        ~columns:[ "name"; "type"; "value" ]
        (List.map
           (fun (n, c) -> [ n; Code.type_text c.ty; value_text c.value ])
           (Utils.Assoc.sorted s.store.statics));
      text_table ~title:"heap"
        ~columns:[ "heap"; "type"; "value" ]
        (cell_rows Heap_id.to_int s.store.heap);
      container ~title:"locals"
        (List.mapi
           (fun i scopes ->
             container
               ~title:("fn frame " ^ string_of_int i)
               (List.mapi
                  (fun j cells ->
                    text_table
                      ~title:("scope " ^ string_of_int j)
                      ~columns:[ "local"; "type"; "value" ]
                      (cell_rows Local_id.to_int cells))
                  scopes))
           s.store.locals);
      indexed_table ~title:"inner_k" ~columns:[ "depth"; "frame" ] inner_text
        s.inner_k;
      text_table ~title:"fn_k"
        ~columns:[ "depth"; "function"; "rets"; "resume" ]
        (List.mapi
           (fun i f ->
             [
               string_of_int i;
               f.function_name;
               string_of_int (List.length f.ret_locs);
               string_of_int (List.length f.inner_k);
             ])
           s.fn_k);
    ]

module Ainput = Parser.Ainput
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.String
module Foutput = Utils.Text.Natural
