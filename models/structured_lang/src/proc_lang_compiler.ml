open Common
open Compiler_common
module P = Proc_lang
module Source = P.Machine
module Target = I.Machine

type varloc = Global | Param of int | Local of int

type scope = {
  locals_len : int;
  params_len : int;
  vars : (string * varloc) list;
  entries : (string * string) list;
}

let index_from_top c offset dst =
  let len = fresh_vreg c "len" in
  [ I.LGet len; I.BinOp (dst, I.VReg len, I.Sub, I.Imm (N.of_int offset)) ]

let place_for_var c scope v =
  match
    Utils.Assoc.lookup v scope.vars ("Unknown variable in compiler: " ^ v)
  with
  | Global -> ([], I.Label v)
  | Local i ->
      let idx = fresh_vreg c "idx_local" in
      (index_from_top c (i + 1) idx, I.SAcc (I.VReg idx))
  | Param i ->
      let idx = fresh_vreg c "idx_param" in
      (index_from_top c (scope.locals_len + 2 + i) idx, I.SAcc (I.VReg idx))

let load_var c scope v dst =
  let stmts, p = place_for_var c scope v in
  stmts @ [ I.Load (dst, p) ]

let store_var c scope v src =
  let stmts, p = place_for_var c scope v in
  stmts @ [ I.Store (p, src) ]

let atom c scope dst = function
  | Imm n -> [ I.Assign (dst, I.Imm n) ]
  | Var v -> load_var c scope v dst

let aexp c scope dst = function
  | Atom a -> atom c scope dst a
  | Bin (a, o, b) ->
      let l = fresh_vreg c "lhs" in
      let r = fresh_vreg c "rhs" in
      atom c scope l a @ atom c scope r b
      @ [ I.BinOp (dst, I.VReg l, op o, I.VReg r) ]

let rec compile_stmt stmt cont scope c =
  match stmt with
  | P.Nop ->
      let label = fresh_region c "nop" in
      single c label [ I.Nop ] (enter cont);
      label
  | P.Assign (v, a) ->
      let label = fresh_region c "assign" in
      let r = fresh_vreg c "assign" in
      let ss = aexp c scope r a in
      let st = store_var c scope v (I.VReg r) in
      single c label (ss @ st) (enter cont);
      label
  | P.Seq (a, b) ->
      let right = compile_stmt b cont scope c in
      compile_stmt a right scope c
  | P.If ((a, rel, b), body) ->
      let yes = compile_stmt body cont scope c in
      let label = fresh_region c "if" in
      let l = fresh_vreg c "if_lhs" in
      let r = fresh_vreg c "if_rhs" in
      branch_region c label
        (atom c scope l a @ atom c scope r b)
        (cond rel (I.VReg l) (I.VReg r))
        yes cont;
      label
  | P.While ((a, rel, b), body) ->
      let head = fresh_region c "while_test" in
      let yes = compile_stmt body head scope c in
      let l = fresh_vreg c "while_lhs" in
      let r = fresh_vreg c "while_rhs" in
      branch_region c head
        (atom c scope l a @ atom c scope r b)
        (cond rel (I.VReg l) (I.VReg r))
        yes cont;
      head
  | P.Return vars ->
      let label = fresh_region c "ret" in
      let regs =
        List.map
          (fun v ->
            let r = fresh_vreg c "retv" in
            (r, load_var c scope v r))
          vars
      in
      let ret_addr = fresh_vreg c "ret_addr" in
      let ss =
        List.concat_map snd regs
        @ List.init scope.locals_len (fun _ ->
            I.Pop (fresh_vreg c "drop_local"))
        @ [ I.Pop ret_addr ]
        @ List.init scope.params_len (fun _ ->
            I.Pop (fresh_vreg c "drop_param"))
        @ List.map (fun (r, _) -> I.Push (I.VReg r)) (List.rev regs)
      in
      single c label ss (I.Enter ([], I.VReg ret_addr));
      label
  | P.Call (name, args, rets) ->
      let ret_region = fresh_region c "call_ret" in
      (* Evaluate all arguments before pushing, while the caller's frame is intact. *)
      let args =
        List.map
          (fun arg ->
            let r = fresh_vreg c "arg" in
            (r, load_var c scope arg r))
          (List.rev args)
      in
      let target =
        Utils.Assoc.lookup name scope.entries
          ("Unknown procedure in compiler: " ^ name)
      in
      let call_region = fresh_region c "call" in
      single c call_region
        (List.concat_map snd args
        @ List.map (fun (r, _) -> I.Push (I.VReg r)) args
        @ [ I.Push (I.CodeLabel ret_region) ])
        (enter target);
      let ret_regs = List.map (fun v -> (v, fresh_vreg c "ret")) rets in
      let ss =
        List.map (fun (_, r) -> I.Pop r) ret_regs
        @ List.concat_map
            (fun (v, r) -> store_var c scope v (I.VReg r))
            ret_regs
      in
      single c ret_region ss (enter cont);
      call_region

let compile (source : P.code) =
  L.protect (fun () ->
      let c = create () in
      let halt = fresh_region c "halt" in
      single c halt [ I.Nop ] (I.Halt []);
      let entries =
        List.map
          (fun (p : P.proc_def) -> (p.name, fresh_region c ("proc_" ^ p.name)))
          source.procs
      in
      List.iter
        (fun (p : P.proc_def) ->
          let vars = List.map (fun v -> (v, Global)) source.statics in
          let vars =
            List.fold_left
              (fun vs (i, v) -> Utils.Assoc.set v (Param i) vs)
              vars
              (List.mapi (fun i v -> (i, v)) p.params)
          in
          let vars =
            List.fold_left
              (fun vs (i, v) -> Utils.Assoc.set v (Local i) vs)
              vars
              (List.mapi (fun i v -> (i, v)) p.locals)
          in
          let scope =
            {
              vars;
              locals_len = List.length p.locals;
              params_len = List.length p.params;
              entries;
            }
          in
          let body =
            compile_stmt (P.with_implicit_return p.body) halt scope c
          in
          single c
            (List.assoc p.name entries)
            (List.map (fun _ -> I.Push (I.Imm N.zero)) p.locals)
            (enter body))
        source.procs;
      let main =
        Utils.Assoc.lookup "main" entries "Procedure 'main' is required"
      in
      single c "main" [ I.Push (I.CodeLabel halt); I.Nop ] (enter main);
      {
        I.statics =
          List.map (fun label -> { I.label; value = N.zero }) source.statics;
        regions = c.regions;
      })

let encode_ainput = Compiler_common.encode_ainput
let encode_rinput s = Ok s
let decode_routput s = Ok s

let decode_foutput values =
  L.protect (fun () ->
      List.map
        (fun (k, v) ->
          match v with
          | I.Num n -> (k, n)
          | _ ->
              failwith
                ("expected numeric global for " ^ k ^ ", got "
               ^ I.Ainput.print_value v))
        (Utils.Assoc.sorted values))

let api =
  Utils.Compiler.make_api
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
