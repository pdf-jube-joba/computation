include Runtime_types
module Code = Code
open Store

let get = Utils_base.Error.get

let expect_number = function
  | Num n -> n
  | v -> failwith ("expected a number, found " ^ Runtime_value.print v)

let rec eval_place s = function
  | Var v ->
      Utils_base.Assoc.lookup v s.env
        ("unknown variable: " ^ Utils_names.Identifier.to_string v)
  | Deref v -> (
      match eval_value s v with
      | Location loc -> loc
      | Null_ptr -> failwith "cannot dereference #null_ptr"
      | Num _ -> failwith "dereference expects a location value")

and eval_value s = function
  | Number n -> Num n
  | NullPtr -> Null_ptr
  | BinOp (a, op, b) ->
      let a = expect_number (eval_value s a)
      and b = expect_number (eval_value s b) in
      Num ((if op = Add then Int64.add else Int64.sub) a b)
  | Load p -> store_lookup s (eval_place s p)
  | Addr p -> Location (eval_place s p)

let validate code =
  Utils_names.Identifier.check_unique
    ~message:(fun s ->
      "duplicate function: " ^ Utils_names.Identifier.to_string s)
    (List.map (fun f -> f.name) code.functions)

let function_lookup c name =
  match List.find_opt (fun f -> f.name = name) c.functions with
  | Some f -> f
  | None ->
      failwith ("unknown function: " ^ Utils_names.Identifier.to_string name)

let split_body = function
  | [] -> failwith "function body must not be empty"
  | s :: ss -> (Some s, ss)

let make code () =
  Utils_base.Error.protect (fun () ->
      validate code;
      let f =
        match
          List.find_opt
            (fun f -> f.name = Utils_names.Identifier.make_exn "main")
            code.functions
        with
        | Some f -> f
        | None -> failwith "function 'main' is missing"
      in
      let current_stmt, inner_k = split_body f.body in
      let frame_id, next_frame_id =
        Frame_id.Supply.fresh Frame_id.Supply.empty
      in
      let env, values =
        bindings frame_id f.params (List.map (fun _ -> Num 0L) f.params)
      in
      {
        code;
        current_function = f.name;
        current_stmt;
        inner_k;
        env = Utils_base.Assoc.sorted env;
        store = [ { frame_id; values } ];
        fn_k = [];
        next_frame_id;
      })

let step s () =
  Utils_base.Error.protect (fun () ->
      let continue next =
        Utils_runtime.Machine.Continue { next; output = "" }
      in
      match s.current_stmt with
      | None -> (
          match s.inner_k with
          | [] -> failwith "control reached end of function body without return"
          | st :: inner_k -> continue { s with current_stmt = Some st; inner_k }
          )
      | Some (Assign (p, v)) ->
          let loc = eval_place s p in
          let value = eval_value s v in
          continue { (store_assign s loc value) with current_stmt = None }
      | Some (Ifz (c, a, b)) ->
          let v = eval_value s c in
          let selected =
            match v with
            | Num 0L -> a
            | Num _ -> b
            | _ ->
                failwith ("ifz expects a number, found " ^ Runtime_value.print v)
          in
          continue { s with current_stmt = Some selected }
      | Some (Call (name, args)) ->
          let values = List.map (eval_value s) args in
          let f = function_lookup s.code name in
          if List.length values <> List.length f.params then
            failwith
              (Printf.sprintf
                 "argument count mismatch for %s: expected %d, got %d"
                 (Utils_names.Identifier.to_string name)
                 (List.length f.params) (List.length values));
          let current_stmt, inner_k = split_body f.body in
          let frame_id, next_frame_id = Frame_id.Supply.fresh s.next_frame_id in
          let env, values = bindings frame_id f.params values in
          continue
            {
              s with
              current_function = f.name;
              current_stmt;
              inner_k;
              env = Utils_base.Assoc.sorted env;
              store = { frame_id; values } :: s.store;
              fn_k =
                {
                  env = s.env;
                  inner_k = s.inner_k;
                  function_name = s.current_function;
                }
                :: s.fn_k;
              next_frame_id;
            }
      | Some Return -> (
          match s.store with
          | [] -> failwith "return on empty store stack"
          | finished :: store -> (
              let s = { s with store } in
              match s.fn_k with
              | [] ->
                  Utils_runtime.Machine.Halt
                    (Output_text.format_final_state s finished)
              | f :: fn_k ->
                  continue
                    {
                      s with
                      current_function = f.function_name;
                      env = f.env;
                      inner_k = f.inner_k;
                      fn_k;
                      current_stmt = None;
                    })))

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      Utils_base.Error.protect (fun () ->
          validate s.code;
          let ids = ref Frame_id.Set.empty in
          let location (l : location) =
            ids := Frame_id.Set.add l.frame_id !ids
          in
          List.iter
            (fun frame ->
              ids := Frame_id.Set.add frame.frame_id !ids;
              List.iter
                (fun (l, v) ->
                  location l;
                  match v with Location l -> location l | _ -> ())
                frame.values)
            s.store;
          List.iter (fun (_, l) -> location l) s.env;
          List.iter
            (fun (frame : call_frame) ->
              List.iter (fun (_, l) -> location l) frame.env)
            s.fn_k;
          get (Frame_id.Supply.validate_used !ids s.next_frame_id);
          s))

let render = View.render

module Ainput = Utils_codec.Text.Unit
module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.String
module Foutput = Utils_codec.Text.String
