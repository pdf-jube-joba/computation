include Types
module Code = Code

let link (code : code) =
  Utils_base.Error.protect (fun () ->
      let block_index =
        List.mapi (fun i (b : block) -> (b.label, i)) code.blocks
      in
      let static_labels =
        List.mapi (fun i (s : static_def) -> (s.label, i)) code.statics
      in
      let unique kind labels =
        Utils_names.Identifier.check_unique
          ~message:(fun label ->
            "Duplicate " ^ kind ^ " label: @"
            ^ Utils_names.Identifier.to_string label)
          (List.map fst labels)
      in
      unique "block" block_index;
      unique "static" static_labels;
      let entry_block =
        match
          List.assoc_opt (Utils_names.Identifier.make_exn "main") block_index
        with
        | Some i -> i
        | None -> failwith "Missing entry block: @main"
      in
      {
        blocks = Array.of_list code.blocks;
        entry_block;
        block_index;
        static_labels;
      })

let make (code : code) ainput =
  Utils_base.Error.protect (fun () ->
      let compiled = Utils_base.Error.get (link code) in
      {
        code;
        compiled;
        current_block = compiled.entry_block;
        vregs = [||];
        memory =
          Array.of_list
            (List.map (fun (s : static_def) -> s.value) code.statics @ ainput);
        halted = false;
      })

let eval_addr s = function
  | VReg i -> Tiny_isa_shared.Memory.read s.vregs i
  | Imm n -> n
  | Label n -> (
      match List.assoc_opt n s.compiled.static_labels with
      | Some i -> N.of_int i
      | None -> (
          match List.assoc_opt n s.compiled.block_index with
          | Some i -> N.of_int i
          | None ->
              failwith ("Unknown label: @" ^ Utils_names.Identifier.to_string n)
          ))

let eval_place s p = Utils_base.Error.get (N.to_int (eval_addr s p))

let eval_value s = function
  | VRegValue i -> Tiny_isa_shared.Memory.read s.vregs i
  | ImmValue n -> n
  | LabelValue n -> eval_addr s (Label n)
  | Deref p -> Tiny_isa_shared.Memory.read s.memory (eval_place s p)

let eval_cond s = function
  | Lt (a, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      N.compare a b < 0
  | Eq (a, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      N.equal a b
  | Gt (a, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      N.compare a b > 0

let step s () =
  Utils_base.Error.protect (fun () ->
      if s.halted then Utils_runtime.Machine.Halt (Array.to_list s.memory)
      else (
        if
          s.current_block < 0
          || s.current_block >= Array.length s.compiled.blocks
        then
          failwith
            (Printf.sprintf "Current block out of range: %d" s.current_block);
        let block = s.compiled.blocks.(s.current_block) in
        let s =
          List.fold_left
            (fun s -> function
              | Assign (dst, src) ->
                  let n = eval_value s src in
                  { s with vregs = Tiny_isa_shared.Memory.write s.vregs dst n }
              | BinOp (dst, a, op, b) ->
                  let a = eval_value s a in
                  let b = eval_value s b in
                  {
                    s with
                    vregs =
                      Tiny_isa_shared.Memory.write s.vregs dst
                        ((match op with Add -> N.add | Sub -> N.sub) a b);
                  }
              | Store (place, src) ->
                  let addr = eval_place s place in
                  let n = eval_value s src in
                  {
                    s with
                    memory = Tiny_isa_shared.Memory.write s.memory addr n;
                  })
            s block.stmts
        in
        let target =
          match List.find_opt (fun j -> eval_cond s j.cond) block.cont.ifs with
          | Some j -> j.target
          | None -> block.cont.jump
        in
        let current_block =
          Utils_base.Error.get (N.to_int (eval_addr s target))
        in
        if current_block < 0 || current_block >= Array.length s.compiled.blocks
        then
          failwith
            (Printf.sprintf "Block address out of range: %d" current_block);
        Utils_runtime.Machine.Continue
          { next = { s with current_block }; output = () }))

let snapshot = state_to_yojson
let restore = state_of_yojson
let render = View.render

module Ainput = Utils_codec.Text.Natural_list
module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Utils_codec.Text.Natural_list
