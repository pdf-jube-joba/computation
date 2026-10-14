open Types
open Resolution
module L = Utils_base.Error

let get = Utils_base.Error.get
let encode = state_to_yojson

let decode json =
  Result.bind (state_of_yojson json) (fun s ->
      L.protect (fun () ->
          validate s.code;
          if s.resolution <> build_resolution s.code then
            failwith "Inconsistent name resolution in snapshot";
          let known_data id = ignore (name_of_id s.resolution.data id) in
          let heap_ids = ref (Heap_id.Set.of_list (List.map fst s.heap)) in
          let check_value = function
            | Num _ -> ()
            | Code_label target -> ignore (target_name s.resolution target)
            | Key (Static id) -> known_data id
            | Key (Heap (id, _)) -> heap_ids := Heap_id.Set.add id !heap_ids
            | Key (Stack _) -> ()
          in
          List.iter
            (fun (id, v) ->
              ignore (name_of_id s.resolution.registers id);
              check_value v)
            s.vregs;
          List.iter
            (fun (id, v) ->
              known_data id;
              check_value v)
            s.static_mem;
          List.iter check_value s.stack;
          List.iter (fun (_, values) -> List.iter check_value values) s.heap;
          get (Heap_id.Supply.validate_used !heap_ids s.next_handle);
          s))
