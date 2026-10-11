open Common
module I = Flow_ir

type ctx = {
  mutable regions : I.region list;
  mutable region_id : int;
  mutable block_id : int;
  mutable vreg_id : int;
}

let create () = { regions = []; region_id = 0; block_id = 0; vreg_id = 0 }

let fresh_region c prefix =
  let i = c.region_id in
  c.region_id <- i + 1;
  prefix ^ "_" ^ string_of_int i

let fresh_block c prefix =
  let i = c.block_id in
  c.block_id <- i + 1;
  prefix ^ "_" ^ string_of_int i

let fresh_vreg c prefix =
  let i = c.vreg_id in
  c.vreg_id <- i + 1;
  prefix ^ "_" ^ string_of_int i

let go label = I.Go ([], I.CodeLabel label)
let enter label = I.Enter ([], I.CodeLabel label)
let block label stmts cont : I.block = { label; stmts; cont }
let region c label blocks = c.regions <- c.regions @ [ { I.label; blocks } ]
let single c label stmts cont = region c label [ block "entry" stmts cont ]

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
  region c label
    [
      block "entry" stmts
        (I.Go
           ( [ { I.cond = condition; target = I.CodeLabel "true" } ],
             I.CodeLabel "false" ));
      block "true" [] (enter yes);
      block "false" [] (enter no);
    ]

let static_defs names =
  List.map
    (fun label -> { I.label; value = N.zero })
    (List.sort_uniq String.compare names)

let encode_ainput e = Ok (List.map (fun (k, n) -> (k, I.Num n)) (sorted e))

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
        (sorted values))

let text_api ~parse_code ~compile ~parse_ainput ~encode_ainput ~print_ainput
    ~parse_rinput ~encode_rinput ~decode_routput ~decode_foutput ~print_foutput
    : Utils.Compiler.api =
  {
    compile_code =
      (fun text ->
        Result.bind (parse_code text) (fun code ->
            Result.map I.Code.print (compile code)));
    encode_ainput =
      (fun text ->
        Result.bind (parse_ainput text) (fun input ->
            Result.map print_ainput (encode_ainput input)));
    encode_rinput = (fun text -> Result.bind (parse_rinput text) encode_rinput);
    decode_routput;
    decode_foutput =
      (fun text ->
        Result.bind (I.Foutput.parse text) (fun output ->
            Result.map print_foutput (decode_foutput output)));
  }

let atom_vars = function Var v -> [ v ] | Imm _ -> []

let aexp_vars = function
  | Atom a -> atom_vars a
  | Bin (a, _, b) -> atom_vars a @ atom_vars b

let bexp_vars (a, _, b) = atom_vars a @ atom_vars b
