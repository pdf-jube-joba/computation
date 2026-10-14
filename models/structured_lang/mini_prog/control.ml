open Runtime_types
open Typed_store
open Eval
open Scope

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
                Utils_base.Assoc.set id
                  { ty; value = default_value ty }
                  s.store.heap;
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
