open Common
open Compiler_common
module E = Expr_lang
module Source = E.Machine
module Target = I.Machine

let entry_first blocks entry =
  match List.find_opt (fun (b : I.block) -> b.label = entry) blocks with
  | None -> failwith ("entry block not found: :" ^ entry)
  | Some b ->
      let first = List.hd blocks in
      List.mapi
        (fun i (candidate : I.block) ->
          if i = 0 then b
          else if candidate.label = entry then first
          else candidate)
        blocks

let add blocks b = blocks := !blocks @ [ b ]

let rec compile_aexp exp dst end_block blocks c =
  match exp with
  | E.Num n ->
      let label = fresh_block c "a_num" in
      add blocks (block label [ I.Assign (dst, I.Imm n) ] (go end_block));
      label
  | E.Var v ->
      let label = fresh_block c "a_var" in
      add blocks (block label [ I.Load (dst, I.Label v) ] (go end_block));
      label
  | E.BinOp (a, o, b) ->
      let l = fresh_vreg c "lhs" in
      let r = fresh_vreg c "rhs" in
      let label = fresh_block c "a_bin" in
      add blocks
        (block label [ I.BinOp (dst, I.VReg l, op o, I.VReg r) ] (go end_block));
      let right = compile_aexp b r label blocks c in
      compile_aexp a l right blocks c
  | E.IfThenElse (cond, a, b) ->
      let yes = compile_aexp a dst end_block blocks c in
      let no = compile_aexp b dst end_block blocks c in
      compile_bexp cond yes no blocks c

and compile_bexp exp yes no blocks c =
  match exp with
  | E.Rel (a, rel, b) ->
      let l = fresh_vreg c "c_lhs" in
      let r = fresh_vreg c "c_rhs" in
      let label = fresh_block c "cond" in
      add blocks
        (block label []
           (I.Go
              ( [
                  {
                    I.cond = cond rel (I.VReg l) (I.VReg r);
                    target = I.CodeLabel yes;
                  };
                ],
                I.CodeLabel no )));
      let right = compile_aexp b r label blocks c in
      compile_aexp a l right blocks c
  | E.Or (a, b) ->
      let right = compile_bexp b yes no blocks c in
      compile_bexp a yes right blocks c
  | E.Not b -> compile_bexp b no yes blocks c

let compile_bexp_region exp yes no c =
  let label = fresh_region c "stmt_cond" in
  let y = fresh_block c "true" in
  let n = fresh_block c "false" in
  let blocks =
    ref [ block y [ I.Nop ] (enter yes); block n [ I.Nop ] (enter no) ]
  in
  let entry = compile_bexp exp y n blocks c in
  region c label (entry_first !blocks entry);
  label

let rec compile_stmt stmt cont c =
  match stmt with
  | E.Nop ->
      let label = fresh_region c "stmt_nop" in
      let entry = fresh_block c "entry" in
      region c label [ block entry [ I.Nop ] (enter cont) ];
      label
  | E.Assign (v, a) ->
      let label = fresh_region c "stmt_assign" in
      let dst = fresh_vreg c "assign" in
      let last = fresh_block c "assign_end" in
      let blocks =
        ref [ block last [ I.Store (I.Label v, I.VReg dst) ] (enter cont) ]
      in
      let entry = compile_aexp a dst last blocks c in
      region c label (entry_first !blocks entry);
      label
  | E.Seq (a, b) ->
      let right = compile_stmt b cont c in
      compile_stmt a right c
  | E.If (cond, b) ->
      let yes = compile_stmt b cont c in
      compile_bexp_region cond yes cont c
  | E.While (cond, b) ->
      let head = fresh_region c "while_head" in
      let body = compile_stmt b head c in
      let test = compile_bexp_region cond body cont c in
      let hb = fresh_block c "head_entry" in
      region c head [ block hb [ I.Nop ] (enter test) ];
      head

let rec a_vars = function
  | E.Var v -> [ v ]
  | E.Num _ -> []
  | E.BinOp (a, _, b) -> a_vars a @ a_vars b
  | E.IfThenElse (c, a, b) -> b_vars c @ a_vars a @ a_vars b

and b_vars = function
  | E.Rel (a, _, b) -> a_vars a @ a_vars b
  | E.Or (a, b) -> b_vars a @ b_vars b
  | E.Not b -> b_vars b

let rec stmt_vars = function
  | E.Nop -> []
  | E.Assign (v, a) -> v :: a_vars a
  | E.Seq (a, b) -> stmt_vars a @ stmt_vars b
  | E.If (c, b) | E.While (c, b) -> b_vars c @ stmt_vars b

let compile source =
  L.protect (fun () ->
      let c = create () in
      let halt = fresh_region c "halt" in
      let hb = fresh_block c "halt_entry" in
      region c halt [ block hb [ I.Nop ] (I.Halt []) ];
      let entry = compile_stmt source halt c in
      {
        I.statics = static_defs (stmt_vars source);
        regions =
          {
            I.label = "main";
            blocks = [ block "entry" [ I.Nop ] (enter entry) ];
          }
          :: c.regions;
      })

let encode_ainput = encode_ainput
let encode_rinput () = Ok ""
let decode_routput _ = Ok ()
let decode_foutput = decode_foutput "expr"

let api =
  text_api ~parse_code:E.Code.parse ~compile ~parse_ainput:E.Ainput.parse
    ~encode_ainput ~print_ainput:I.Ainput.print ~parse_rinput:E.Rinput.parse
    ~encode_rinput
    ~decode_routput:(fun _ -> Ok "")
    ~decode_foutput ~print_foutput:Env.print
