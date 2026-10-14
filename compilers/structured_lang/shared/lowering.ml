open Structured_lang_shared.Arithmetic
module I = Flow_ir.Main
include Flow_ir.Builder

let cond rel a b =
  match rel with Lt -> I.Lt (a, b) | Eq -> I.Eq (a, b) | Gt -> I.Gt (a, b)

let op = function Add -> I.Add | Sub -> I.Sub

let emit_atom dst = function
  | Var v -> [ I.Load (dst, I.Label v) ]
  | Imm n -> [ I.Assign (dst, I.Imm n) ]

let emit_aexp c dst = function
  | Atom a -> emit_atom dst a
  | Bin (a, o, b) ->
      let l = fresh_vreg c "lhs" in
      let r = fresh_vreg c "rhs" in
      emit_atom l a @ emit_atom r b
      @ [ I.BinOp (dst, I.VReg l, op o, I.VReg r) ]

(* Conditional continuations always jump to a local block; that block can enter
   another region. The Rust compilers emitted region labels directly here. *)
let branch_region c label stmts condition yes no =
  let entry = fresh_block c "entry" in
  let true_block = fresh_block c "true" in
  let false_block = fresh_block c "false" in
  region c label
    [
      block entry stmts
        (I.Go
           ( [ { I.cond = condition; target = I.CodeLabel true_block } ],
             I.CodeLabel false_block ));
      block true_block [] (enter yes);
      block false_block [] (enter no);
    ]

let static_defs names =
  List.map
    (fun label -> { I.label; value = N.zero })
    (List.sort_uniq String.compare names)

let encode_ainput e =
  Ok (List.map (fun (k, n) -> (k, I.Num n)) (Utils_base.Assoc.sorted e))

let decode_foutput model values =
  L.protect (fun () ->
      List.map
        (fun (k, v) ->
          let n =
            match v with
            | I.Num n -> n
            | _ ->
                failwith
                  (model ^ " decode expects numeric static value for " ^ k
                 ^ ", got " ^ I.Ainput.print_value v)
          in
          ignore (get (Utils_names.Identifier.make k));
          (k, n))
        (Utils_base.Assoc.sorted values))
