include Types
module Ainput = Input_text
module Code = Code
open Resolution
module L = Utils_base.Error

let get = Utils_base.Error.get
let runtime_text = View.runtime_text
let register_id s name = id_of_name s.resolution.registers name

let read_place s = function
  | Static l -> (
      match List.assoc_opt l s.static_mem with
      | Some v -> v
      | None ->
          failwith
            ("Unknown data label: @"
            ^ Utils_names.Identifier.to_string (name_of_id s.resolution.data l)
            ))
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

let make code input =
  Utils_base.Error.protect (fun () ->
      validate code;
      let resolution = build_resolution code in
      let rec main_index i = function
        | [] -> assert false
        | (r : region) :: rest ->
            if r.label = Utils_names.Identifier.make_exn "main" then i
            else main_index (i + 1) rest
      in
      let main = main_index 0 code.regions in
      let mem =
        List.map
          (fun (s : static_def) ->
            (id_of_name resolution.data s.label, Num s.value))
          code.statics
      in
      let mem =
        List.fold_left
          (fun m (k, v) ->
            let key = id_of_name resolution.data k in
            Utils_base.Assoc.set key (resolve_input resolution main v) m)
          mem input
      in
      let rec idx i = function
        | [] -> assert false
        | (r : region) :: rs ->
            if r.label = Utils_names.Identifier.make_exn "main" then i
            else idx (i + 1) rs
      in
      {
        code;
        resolution;
        current_region = idx 0 code.regions;
        current_block = 0;
        current_line = 0;
        vregs = [];
        static_mem = mem;
        stack = [];
        heap = [];
        next_handle = Heap_id.Supply.after (get (Heap_id.of_int 0));
        halted = false;
      })

let reg s r =
  Option.value (List.assoc_opt (register_id s r) s.vregs) ~default:(Num N.zero)

let num s = function
  | Num n -> n
  | v -> failwith ("Expected number, got " ^ runtime_text s v)

let index s v = get (N.to_int (num s v))

let rec eval_value s = function
  | VReg r -> reg s r
  | Imm n -> Num n
  | CodeLabel l -> Code_label (resolve_target s.resolution s.current_region l)
  | Ref p -> Key (eval_place s p)

and eval_place s = function
  | Label l ->
      let id = id_of_name s.resolution.data l in
      if List.mem_assoc id s.static_mem then Static id
      else
        failwith
          ("Place is not data label: @" ^ Utils_names.Identifier.to_string l)
  | Deref r -> (
      match reg s r with
      | Key k -> k
      | v ->
          failwith ("deref expects place key in vreg, got " ^ runtime_text s v))
  | SAcc v -> Stack (index s (eval_value s v))
  | HAcc (h, i) ->
      Heap
        ( get (Heap_id.of_int (index s (eval_value s h))),
          index s (eval_value s i) )

let eval_cond s c =
  let a, b, op =
    match c with
    | Lt (a, b) -> (a, b, -1)
    | Eq (a, b) -> (a, b, 0)
    | Gt (a, b) -> (a, b, 1)
  in
  let a = eval_value s a and b = eval_value s b in
  match (a, b) with
  | Num a, Num b ->
      let cmp = N.compare a b in
      if op = 0 then cmp = 0 else if op < 0 then cmp < 0 else cmp > 0
  | _ when op = 0 -> a = b
  | _ ->
      failwith
        ("Condition "
        ^ (if op < 0 then "lt" else "gt")
        ^ " supports only numbers, got " ^ runtime_text s a ^ " and "
        ^ runtime_text s b)

let find_idx f xs err =
  let rec loop i = function
    | [] -> failwith err
    | x :: xs -> if f x then i else loop (i + 1) xs
  in
  loop 0 xs

let target s v =
  match eval_value s v with
  | Code_label l -> l
  | x -> failwith ("jump target expects code label, got " ^ runtime_text s x)

let jump_go s v =
  let label =
    match v with
    | CodeLabel name -> local_target s.resolution s.current_region name
    | _ -> target s v
  in
  let id =
    match label with
    | Block_target id -> id
    | _ -> failwith "goto expects a local block ID"
  in
  {
    s with
    current_block =
      find_idx
        (fun (_, value) -> Block_id.equal id value)
        (List.nth s.resolution.blocks s.current_region)
        "goto target is not in the current region";
    current_line = 0;
  }

let jump_enter s v =
  let label =
    match v with
    | CodeLabel name -> region_target s.resolution name
    | _ -> target s v
  in
  let id =
    match label with
    | Region_target id -> id
    | _ -> failwith "enter expects a region ID"
  in
  {
    s with
    current_region =
      find_idx
        (fun (_, value) -> Region_id.equal id value)
        s.resolution.regions "unknown region ID";
    current_block = 0;
    current_line = 0;
    vregs = [];
  }

let step s input =
  Utils_base.Error.protect (fun () ->
      let halt s =
        Utils_runtime.Machine.Halt
          (Utils_base.Assoc.sorted
             (List.map
                (fun (id, v) ->
                  (name_of_id s.resolution.data id, named_value s.resolution v))
                s.static_mem))
      in
      let continue s o =
        Utils_runtime.Machine.Continue { next = s; output = o }
      in
      if s.halted then halt s
      else
        let r =
          Utils_base.List_ext.nth s.code.regions s.current_region
            "Current region out of range"
        in
        let b =
          Utils_base.List_ext.nth r.blocks s.current_block
            "Current block out of range"
        in
        let sl = List.length b.stmts and il = List.length (ifs b.cont) in
        if s.current_line < sl then
          let st = List.nth b.stmts s.current_line in
          let s = { s with current_line = s.current_line + 1 } in
          let put r v =
            { s with vregs = Utils_base.Assoc.set (register_id s r) v s.vregs }
          in
          let s, o =
            match st with
            | Nop -> (s, "")
            | Assign (r, v) -> (put r (eval_value s v), "")
            | Load (r, p) -> (put r (read_place s (eval_place s p)), "")
            | BinOp (r, a, op, b) ->
                let a = num s (eval_value s a) and b = num s (eval_value s b) in
                (put r (Num ((if op = Add then N.add else N.sub) a b)), "")
            | Store (p, v) ->
                (write_place s (eval_place s p) (eval_value s v), "")
            | Push v -> ({ s with stack = s.stack @ [ eval_value s v ] }, "")
            | Pop r -> (
                match List.rev s.stack with
                | [] -> failwith "pop on empty stack"
                | v :: vs ->
                    ( {
                        s with
                        stack = List.rev vs;
                        vregs = Utils_base.Assoc.set (register_id s r) v s.vregs;
                      },
                      "" ))
            | LGet r -> (put r (Num (N.of_int (List.length s.stack))), "")
            | HAlloc (v, r) ->
                let size = index s (eval_value s v) in
                let handle, next_handle = Heap_id.Supply.fresh s.next_handle in
                ( {
                    s with
                    heap =
                      Utils_base.Assoc.set handle
                        (List.init size (fun _ -> Num N.zero))
                        s.heap;
                    next_handle;
                    vregs =
                      Utils_base.Assoc.set (register_id s r)
                        (Num (N.of_int (Heap_id.to_int handle)))
                        s.vregs;
                  },
                  "" )
            | HFree v ->
                let h = get (Heap_id.of_int (index s (eval_value s v))) in
                if not (List.mem_assoc h s.heap) then
                  failwith
                    (Printf.sprintf "hfree unknown handle: %d"
                       (Heap_id.to_int h));
                ({ s with heap = List.remove_assoc h s.heap }, "")
            | Print r ->
                let bytes = N.to_bytes (num s (reg s r)) in
                if not (Utils_base.Utf8.valid bytes) then
                  failwith "print byte is not valid single-byte UTF-8";
                (s, bytes)
            | Input p ->
                (write_place s (eval_place s p) (Num (N.of_bytes input)), "")
          in
          continue s o
        else if s.current_line < sl + il then
          let j = List.nth (ifs b.cont) (s.current_line - sl) in
          let s = { s with current_line = s.current_line + 1 } in
          continue (if eval_cond s j.cond then jump_go s j.target else s) ""
        else if s.current_line = sl + il then
          match b.cont with
          | Go (_, v) -> continue (jump_go s v) ""
          | Enter (_, v) -> continue (jump_enter s v) ""
          | Halt _ -> halt s
        else failwith "Current line out of range")

let snapshot = state_to_yojson

let restore json =
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

let render = View.render

module Rinput = Utils_codec.Text.String
module Routput = Utils_codec.Text.String
module Foutput = Ainput
