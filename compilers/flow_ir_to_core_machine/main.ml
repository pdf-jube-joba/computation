module Source = Flow_ir.Main
module Target = Core_machine.Main
module F = Source
module C = Target
module R = Flow_ir.Resolution
module N = Utils_base.Number
module Id = Utils_names.Identifier

let id = Id.make_exn
let imm n = C.Imm (N.of_int n)
let literal n = C.Imm n
let name n = literal (Codec.name n)
let error_label = id "runtime_error"
let output_label = id "output"
let sp = id "stack_size"
let hp = id "heap_end"
let next_handle = id "next_handle"

type triple = C.value_expr * C.value_expr * C.value_expr

type builder = {
  mutable serial : int;
  mutable label : Id.t;
  mutable stmts : C.stmt list;
  mutable blocks : C.block list;
}

let fresh c prefix =
  c.serial <- c.serial + 1;
  id (prefix ^ string_of_int c.serial)

let emit c s = c.stmts <- s :: c.stmts

let finish c ifs ending =
  c.blocks <-
    { C.label = c.label; stmts = List.rev c.stmts; ifs; ending } :: c.blocks;
  c.stmts <- []

let go c l = finish c [] (C.Goto (C.Label l))

let start c l =
  c.label <- l;
  c.stmts <- []

let bind c rhs =
  let r = fresh c "tmp" in
  emit c (C.Assign (r, rhs));
  C.Reg r

let set c r v = emit c (C.Assign (r, C.Value v))
let binary c a op b = bind c (C.Binop (a, op, b))

let branch c cond yes no =
  finish c [ { C.cond; target = C.Label yes } ] (C.Goto (C.Label no))

let require c cond =
  let ok = fresh c "checked" in
  branch c cond ok error_label;
  start c ok

let times3 c n =
  let twice = binary c n C.Add n in
  binary c twice C.Add n

let addr c m i = bind c (C.Addr (C.Memory m, i))

let load c m i =
  let p = addr c m i in
  bind c (C.Load (C.Deref p))

let store c m i v =
  let p = addr c m i in
  emit c (C.Store (C.Deref p, v))

let offset c i n = if n = 0 then i else binary c i C.Add (imm n)

let read3 c m i =
  let t = load c m i in
  let a = load c m (offset c i 1) in
  let b = load c m (offset c i 2) in
  (t, a, b)

let write3 c m i (t, a, b) =
  store c m i t;
  store c m (offset c i 1) a;
  store c m (offset c i 2) b

let regs r =
  let n = N.to_string (Codec.name r) in
  (id ("v" ^ n ^ "t"), id ("v" ^ n ^ "a"), id ("v" ^ n ^ "b"))

let reg_value r =
  let t, a, b = regs r in
  (C.Reg t, C.Reg a, C.Reg b)

let copy c r (t, a, b) =
  let x, y, z = regs r in
  set c x t;
  set c y a;
  set c z b

let numeric c (t, a, _) =
  require c (t, C.Eq, imm 0);
  a

let index c v =
  let n = numeric c v in
  require c (n, C.Lt, literal (N.succ (N.of_int max_int)));
  n

let choose c key cases action =
  let join = fresh c "join" in
  let cases = List.map (fun (v, x) -> (v, x, fresh c "case")) cases in
  finish c
    (List.map
       (fun (v, _, l) -> { C.cond = (key, C.Eq, v); target = C.Label l })
       cases)
    (C.Goto (C.Label error_label));
  List.iter
    (fun (_, x, l) ->
      start c l;
      action x;
      go c join)
    cases;
  start c join

let block_label ri bi = id (Printf.sprintf "block_%d_%d" ri bi)
let region_label ri = id (Printf.sprintf "region_%d" ri)

let compile source =
  Utils_base.Error.protect (fun () ->
      let initial = Utils_base.Error.get (F.make source []) in
      let resolution = initial.F.resolution in
      let c = { serial = 0; label = id "main"; stmts = []; blocks = [] } in
      let statics =
        List.mapi
          (fun i (s : F.static_def) -> (s.label, i, s.value))
          source.F.statics
      in
      let static_cases = List.map (fun (l, i, _) -> (name l, i)) statics in
      let canonical target = name (R.target_name resolution target) in
      let static_slot n = imm (3 * n) in
      let check_static key = choose c key static_cases (fun _ -> ()) in
      let static_addr key =
        let out = fresh c "static_offset" in
        choose c key static_cases (fun i -> set c out (static_slot i));
        C.Reg out
      in
      let rec value ri = function
        | F.Imm n -> (imm 0, literal n, imm 0)
        | F.VReg r ->
            (* Snapshot operands before assigning any destination register. *)
            let t, a, b = reg_value r in
            (bind c (C.Value t), bind c (C.Value a), bind c (C.Value b))
        | F.CodeLabel l ->
            (imm 1, canonical (R.resolve_target resolution ri l), imm 0)
        | F.Ref p -> place ri p
      and place ri = function
        | F.Label l ->
            ignore (R.id_of_name resolution.data l);
            (imm 2, name l, imm 0)
        | F.Deref r ->
            let t, a, b = value ri (F.VReg r) in
            require c (t, C.Gt, imm 1);
            require c (t, C.Lt, imm 5);
            (t, a, b)
        | F.SAcc v ->
            let i = index c (value ri v) in
            (imm 3, i, imm 0)
        | F.HAcc (h, i) ->
            let h = index c (value ri h) in
            require c (h, C.Lt, imm max_int);
            let i = index c (value ri i) in
            (imm 4, h, i)
      in
      let heap_info h =
        require c (h, C.Lt, imm max_int);
        let slot = times3 c h in
        let active, base, len = read3 c 4 slot in
        require c (active, C.Eq, imm 1);
        (base, len, slot)
      in
      let access key action =
        let t, a, b = key in
        choose c t
          [ (imm 2, 2); (imm 3, 3); (imm 4, 4) ]
          (function
            | 2 ->
                let i = static_addr a in
                action 3 i
            | 3 ->
                require c (a, C.Lt, C.Reg sp);
                let i = times3 c a in
                action 1 i
            | _ ->
                let base, len, _ = heap_info a in
                require c (b, C.Lt, len);
                let i = times3 c b in
                let i = binary c base C.Add i in
                action 2 i)
      in
      let load_key key =
        let t = fresh c "loaded_tag"
        and a = fresh c "loaded_a"
        and b = fresh c "loaded_b" in
        access key (fun m i ->
            let x, y, z = read3 c m i in
            set c t x;
            set c a y;
            set c b z);
        (C.Reg t, C.Reg a, C.Reg b)
      in
      let store_key key v = access key (fun m i -> write3 c m i v) in
      let targets enter ri =
        if enter then
          List.mapi
            (fun i (_, r) -> (canonical (F.Region_target r), region_label i))
            resolution.regions
        else
          List.mapi
            (fun bi (_, b) -> (canonical (F.Block_target b), block_label ri bi))
            (List.nth resolution.blocks ri)
      in
      let jump ri enter v =
        let t, a, _ =
          match v with
          | F.CodeLabel l ->
              ( imm 1,
                canonical
                  (if enter then R.region_target resolution l
                   else R.local_target resolution ri l),
                imm 0 )
          | _ -> value ri v
        in
        require c (t, C.Eq, imm 1);
        finish c
          (List.map
             (fun (key, l) -> { C.cond = (a, C.Eq, key); target = C.Label l })
             (targets enter ri))
          (C.Goto (C.Label error_label))
      in
      let condition ri cond yes no =
        let a, op, b =
          match cond with
          | F.Eq (a, b) -> (a, C.Eq, b)
          | F.Lt (a, b) -> (a, C.Lt, b)
          | F.Gt (a, b) -> (a, C.Gt, b)
        in
        let ta, aa, ba = value ri a in
        let tb, ab, bb = value ri b in
        if op = C.Eq then (
          let fields = fresh c "equal_fields" in
          branch c (ta, C.Eq, tb) fields no;
          start c fields;
          let last = fresh c "equal_last" in
          branch c (aa, C.Eq, ab) last no;
          start c last;
          branch c (ba, C.Eq, bb) yes no)
        else
          let a = numeric c (ta, aa, ba) in
          let b = numeric c (tb, ab, bb) in
          branch c (a, op, b) yes no
      in
      let statement ri = function
        | F.Nop -> ()
        | F.Assign (r, v) ->
            let v = value ri v in
            copy c r v
        | F.Load (r, p) ->
            let key = place ri p in
            let v = load_key key in
            copy c r v
        | F.Store (p, v) ->
            let key = place ri p in
            let v = value ri v in
            store_key key v
        | F.BinOp (r, a, op, b) ->
            let a = numeric c (value ri a) in
            let b = numeric c (value ri b) in
            let n = binary c a (if op = F.Add then C.Add else C.Sub) b in
            copy c r (imm 0, n, imm 0)
        | F.Push v ->
            let v = value ri v in
            let i = times3 c (C.Reg sp) in
            write3 c 1 i v;
            emit c (C.Assign (sp, C.Binop (C.Reg sp, C.Add, imm 1)))
        | F.Pop r ->
            require c (C.Reg sp, C.Gt, imm 0);
            emit c (C.Assign (sp, C.Binop (C.Reg sp, C.Sub, imm 1)));
            let i = times3 c (C.Reg sp) in
            let v = read3 c 1 i in
            copy c r v
        | F.LGet r -> copy c r (imm 0, C.Reg sp, imm 0)
        | F.HAlloc (v, r) ->
            let len = index c (value ri v) in
            require c (C.Reg next_handle, C.Lt, imm max_int);
            let h = bind c (C.Value (C.Reg next_handle)) in
            let slot = times3 c h in
            write3 c 4 slot (imm 1, C.Reg hp, len);
            let size = times3 c len in
            emit c (C.Assign (hp, C.Binop (C.Reg hp, C.Add, size)));
            emit c
              (C.Assign (next_handle, C.Binop (C.Reg next_handle, C.Add, imm 1)));
            copy c r (imm 0, h, imm 0)
        | F.HFree v ->
            let h = index c (value ri v) in
            let _, _, slot = heap_info h in
            store c 4 slot (imm 0)
        | F.Print r ->
            let n = numeric c (value ri (F.VReg r)) in
            emit c (C.Print n)
        | F.Input p ->
            let key = place ri p in
            access key (fun m i ->
                let p = addr c m (offset c i 1) in
                emit c (C.Input (C.Deref p));
                store c m i (imm 0);
                store c m (offset c i 2) (imm 0))
      in
      (* Initialize statics, then apply code-independent input records. *)
      set c sp (imm 0);
      set c hp (imm 0);
      set c next_handle (imm 1);
      List.iter
        (fun (_, i, n) -> write3 c 3 (static_slot i) (imm 0, literal n, imm 0))
        statics;
      let count = load c 0 (imm 0) in
      let cursor = fresh c "input_cursor" in
      set c cursor (imm 0);
      let loop = fresh c "input_loop"
      and body = fresh c "input_body"
      and ready = fresh c "ready" in
      go c loop;
      start c loop;
      branch c (C.Reg cursor, C.Lt, count) body ready;
      start c body;
      let twice = binary c (C.Reg cursor) C.Add (C.Reg cursor) in
      let off = binary c twice C.Add twice in
      let off = offset c off 1 in
      let key = load c 0 off in
      let t, a, b = read3 c 0 (offset c off 1) in
      let payload = fresh c "input_payload" in
      set c payload a;
      let main_region = initial.F.current_region in
      let aliases =
        List.map fst resolution.regions
        @ List.map fst (List.nth resolution.blocks main_region)
        @ List.map snd resolution.label_names
        |> List.sort_uniq Id.compare
      in
      choose c t
        [ (imm 0, 0); (imm 1, 1); (imm 2, 2); (imm 3, 3); (imm 4, 4) ]
        (function
          | 1 ->
              choose c a
                (List.map
                   (fun l ->
                     (name l, R.resolve_target resolution main_region l))
                   aliases)
                (fun target -> set c payload (canonical target))
          | 2 -> check_static a
          | _ -> ());
      let slot = static_addr key in
      write3 c 3 slot (t, C.Reg payload, b);
      emit c (C.Assign (cursor, C.Binop (C.Reg cursor, C.Add, imm 1)));
      go c loop;
      start c ready;
      go c (region_label main_region);
      List.iteri
        (fun ri (region : F.region) ->
          start c (region_label ri);
          List.iter
            (fun (r, _) -> copy c r (imm 0, imm 0, imm 0))
            resolution.registers;
          go c (block_label ri 0);
          List.iteri
            (fun bi (block : F.block) ->
              start c (block_label ri bi);
              List.iter (statement ri) block.stmts;
              List.iter
                (fun (j : F.jump_if) ->
                  let yes = fresh c "taken" and no = fresh c "not_taken" in
                  condition ri j.cond yes no;
                  start c yes;
                  jump ri false j.target;
                  start c no)
                (F.ifs block.cont);
              match block.cont with
              | F.Go (_, v) -> jump ri false v
              | F.Enter (_, v) -> jump ri true v
              | F.Halt _ -> go c output_label)
            region.blocks)
        source.regions;
      start c output_label;
      store c 0 (imm 0) (imm (List.length statics));
      List.iter
        (fun (l, i, _) ->
          let v = read3 c 3 (static_slot i) in
          store c 0 (imm (1 + (4 * i))) (name l);
          write3 c 0 (imm (2 + (4 * i))) v)
        statics;
      finish c [] C.Halt;
      start c error_label;
      ignore (bind c (C.Len (C.Memory 0)));
      finish c [] C.Halt;
      { C.memories = 5; blocks = List.rev c.blocks })

let encode_ainput = Codec.encode
let encode_rinput s = Ok s
let decode_routput s = Ok s
let decode_foutput = Codec.decode
