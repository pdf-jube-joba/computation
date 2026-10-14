open Structured_lang_shared.Arithmetic
open Runtime_types
open Validation

let encode = state_to_yojson

let decode json =
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
