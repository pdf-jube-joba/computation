include Runtime_types
module Code = Parser.Code
module L = Utils_base.Error
open Typed_store
open Scope
open Control

let get = Utils_base.Error.get

let make code input =
  Utils_base.Error.protect (fun () ->
      validate code;
      let statics =
        List.fold_left
          (fun ss (d : static_decl) ->
            Utils_base.Assoc.set d.name
              { ty = d.ty; value = default_value d.ty }
              ss)
          [] code.statics
      in
      let s =
        {
          code;
          current_function = Utils_names.Identifier.make_exn "main";
          exec = Ctrl Next;
          env = [];
          store = { statics; heap = []; locals = [] };
          inner_k = [];
          fn_k = [];
          next_local_id = Local_id.Supply.empty;
          next_heap_id = Heap_id.Supply.empty;
        }
      in
      enter_function s
        (Utils_names.Identifier.make_exn "main")
        (List.map (fun n -> Number_value n) input))

let step s () =
  Utils_base.Error.protect (fun () ->
      let continue next =
        Utils_runtime.Machine.Continue { next; output = "" }
      in
      match s.exec with
      | Eval stmt -> continue (step_eval s stmt)
      | Ctrl control ->
          if s.fn_k = [] && s.inner_k = [] && control = Next && s.env = [] then
            failwith "main finished without return";
          continue (step_ctrl s control)
      | Ret values ->
          if s.fn_k = [] then
            match values with
            | [ Number_value n ] -> Utils_runtime.Machine.Halt n
            | [ _ ] -> failwith "main must return a natural number"
            | _ -> failwith "main must return exactly one value"
          else continue (step_ret s values))

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
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
end

let render = View.render

module Ainput = Parser.Ainput
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.String
module Foutput = Utils_data.Text.Natural
