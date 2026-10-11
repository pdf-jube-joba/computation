open Common
open Compiler_common
module C = Internal_ctrl
module Source = C.Machine
module Target = I.Machine

let rec compile_stmt stmt cont loops c =
  match stmt with
  | C.Nop ->
      let label = fresh_region c "nop" in
      single c label [ I.Nop ] (enter cont);
      label
  | C.Seq (a, b) ->
      let right = compile_stmt b cont loops c in
      compile_stmt a right loops c
  | C.Assign (v, a) ->
      let label = fresh_region c "assign" in
      let dst = fresh_vreg c "assign" in
      single c label
        (emit_aexp c dst a @ [ I.Store (I.Label v, I.VReg dst) ])
        (enter cont);
      label
  | C.If ((a, rel, b), body) ->
      let yes = compile_stmt body cont loops c in
      let label = fresh_region c "if" in
      let l = fresh_vreg c "if_lhs" in
      let r = fresh_vreg c "if_rhs" in
      branch_region c label
        (emit_atom l a @ emit_atom r b)
        (cond rel (I.VReg l) (I.VReg r))
        yes cont;
      label
  | C.Break (l, v) ->
      let _, exit = Utils.Assoc.lookup l loops ("unknown loop label: " ^ l) in
      let label = fresh_region c "break" in
      let r = fresh_vreg c "break" in
      single c label [ I.Load (r, I.Label v); I.Push (I.VReg r) ] (enter exit);
      label
  | C.Continue l ->
      let head, _ = Utils.Assoc.lookup l loops ("unknown loop label: " ^ l) in
      let label = fresh_region c "continue" in
      single c label [ I.Nop ] (enter head);
      label
  | C.Loop (l, b, out) ->
      let head = fresh_region c "loop_head" in
      let exit = fresh_region c "loop_exit" in
      let body = compile_stmt b head (Utils.Assoc.set l (head, exit) loops) c in
      single c head [ I.Nop ] (enter body);
      let r = fresh_vreg c "loop_break" in
      single c exit [ I.Pop r; I.Store (I.Label out, I.VReg r) ] (enter cont);
      head

let rec stmt_vars = function
  | C.Nop | C.Continue _ -> []
  | C.Seq (a, b) -> stmt_vars a @ stmt_vars b
  | C.Assign (v, a) -> v :: aexp_vars a
  | C.If (c, b) -> bexp_vars c @ stmt_vars b
  | C.Break (_, v) -> [ v ]
  | C.Loop (_, b, v) -> v :: stmt_vars b

let compile (source : C.code) =
  L.protect (fun () ->
      let c = create () in
      let halt = fresh_region c "halt" in
      let hb = fresh_block c "halt_entry" in
      region c halt [ block hb [ I.Nop ] (I.Halt []) ];
      let entry = compile_stmt source.body halt [] c in
      {
        I.statics = static_defs (source.statics @ stmt_vars source.body);
        regions =
          {
            I.label = "main";
            blocks = [ block "entry" [ I.Nop ] (enter entry) ];
          }
          :: c.regions;
      })

let encode_ainput = Compiler_common.encode_ainput
let encode_rinput () = Ok ""
let decode_routput _ = Ok ""
let decode_foutput = decode_foutput "internal_ctrl"

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
