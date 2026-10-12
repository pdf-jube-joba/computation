open Structured_lang_shared.Arithmetic
module I = Flow_ir.Main
module Region_id = Utils.Id.Make ()
module Block_id = Utils.Id.Make ()
module Vreg_id = Utils.Id.Make ()
module Region_names = Utils.Naming.Make (Region_id)
module Block_names = Utils.Naming.Make (Block_id)
module Vreg_names = Utils.Naming.Make (Vreg_id)

type ctx = {
  mutable regions : I.region list;
  region_supply : Region_id.Supply.t ref;
  block_supply : Block_id.Supply.t ref;
  vreg_supply : Vreg_id.Supply.t ref;
  mutable region_names : Region_names.t;
  mutable block_names : Block_names.t;
  mutable vreg_names : Vreg_names.t;
}

let create () =
  {
    regions = [];
    region_supply = ref Region_id.Supply.empty;
    block_supply = ref Block_id.Supply.empty;
    vreg_supply = ref Vreg_id.Supply.empty;
    region_names = Region_names.reserve "main" Region_names.empty;
    block_names = Block_names.empty;
    vreg_names = Vreg_names.empty;
  }

let fresh_region c hint =
  let id = Region_id.Supply.take c.region_supply in
  let name, next = Region_names.assign ~hint id c.region_names in
  c.region_names <- next;
  name

let fresh_block c hint =
  let id = Block_id.Supply.take c.block_supply in
  let name, next = Block_names.assign ~hint id c.block_names in
  c.block_names <- next;
  name

let fresh_vreg c hint =
  let id = Vreg_id.Supply.take c.vreg_supply in
  let name, next = Vreg_names.assign ~hint id c.vreg_names in
  c.vreg_names <- next;
  name

let go label = I.Go ([], I.CodeLabel label)
let enter label = I.Enter ([], I.CodeLabel label)
let block label stmts cont : I.block = { label; stmts; cont }
let region c label blocks = c.regions <- c.regions @ [ { I.label; blocks } ]

let single c label stmts cont =
  region c label [ block (fresh_block c "entry") stmts cont ]

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
  Ok (List.map (fun (k, n) -> (k, I.Num n)) (Utils.Assoc.sorted e))

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
          ignore (get (Utils.Identifier.make k));
          (k, n))
        (Utils.Assoc.sorted values))

let atom_vars = function Var v -> [ v ] | Imm _ -> []

let aexp_vars = function
  | Atom a -> atom_vars a
  | Bin (a, _, b) -> atom_vars a @ atom_vars b

let bexp_vars (a, _, b) = atom_vars a @ atom_vars b
