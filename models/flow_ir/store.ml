open Types
open Resolution

let read_place s = function
  | Static l -> (
      match List.assoc_opt l s.static_mem with
      | Some v -> v
      | None ->
          failwith ("Unknown data label: @" ^ name_of_id s.resolution.data l))
  | Stack i ->
      Utils_base.List_ext.nth s.stack i
        (Printf.sprintf "Invalid stack key: s[%d]" i)
  | Heap (h, i) ->
      Utils_base.List_ext.nth
        (Option.value (List.assoc_opt h s.heap) ~default:[])
        i
        (Printf.sprintf "Invalid heap key: h[%d][%d]" (Heap_id.to_int h) i)

let write_place s key value =
  match key with
  | Static l ->
      ignore (read_place s key);
      { s with static_mem = Utils_base.Assoc.set l value s.static_mem }
  | Stack i ->
      {
        s with
        stack =
          Utils_base.List_ext.replace s.stack i value
            (Printf.sprintf "Invalid stack key: s[%d]" i);
      }
  | Heap (h, i) ->
      let area =
        match List.assoc_opt h s.heap with
        | Some a -> a
        | None ->
            failwith
              (Printf.sprintf "Unknown heap handle: %d" (Heap_id.to_int h))
      in
      {
        s with
        heap =
          Utils_base.Assoc.set h
            (Utils_base.List_ext.replace area i value
               (Printf.sprintf "Invalid heap key: h[%d][%d]" (Heap_id.to_int h)
                  i))
            s.heap;
      }
