open Structured_lang_compiler_shared.Lowering
module C = Internal_ctrl.Main
module Source = C
module Target = I

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
      let _, exit =
        Utils_base.Assoc.lookup l loops
          ("unknown loop label: " ^ Utils_names.Identifier.to_string l)
      in
      let label = fresh_region c "break" in
      let r = fresh_vreg c "break" in
      single c label [ I.Load (r, I.Label v); I.Push (I.VReg r) ] (enter exit);
      label
  | C.Continue l ->
      let head, _ =
        Utils_base.Assoc.lookup l loops
          ("unknown loop label: " ^ Utils_names.Identifier.to_string l)
      in
      let label = fresh_region c "continue" in
      single c label [ I.Nop ] (enter head);
      label
  | C.Loop (l, b, out) ->
      let head = fresh_region c "loop_head" in
      let exit = fresh_region c "loop_exit" in
      let body =
        compile_stmt b head (Utils_base.Assoc.set l (head, exit) loops) c
      in
      single c head [ I.Nop ] (enter body);
      let r = fresh_vreg c "loop_break" in
      single c exit [ I.Pop r; I.Store (I.Label out, I.VReg r) ] (enter cont);
      head

let compile (source : C.code) =
  Utils_base.Error.protect (fun () ->
      let c = create () in
      let halt = fresh_region c "halt" in
      let hb = fresh_block c "halt_entry" in
      region c halt [ block hb [ I.Nop ] (I.Halt []) ];
      let entry = compile_stmt source.body halt [] c in
      finish
        ~prefix:
          [
            {
              I.label = Utils_names.Identifier.make_exn "main";
              blocks =
                [
                  block
                    (Utils_names.Identifier.make_exn "entry")
                    [ I.Nop ] (enter entry);
                ];
            };
          ]
        c
        (static_defs (source.statics @ C.stmt_vars source.body)))

let encode_ainput = Structured_lang_compiler_shared.Lowering.encode_ainput
let encode_rinput () = Ok ""
let decode_routput _ = Ok ""
let decode_foutput = decode_foutput "internal_ctrl"

let api =
  Utils_runtime.Compiler_text_api.make
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
