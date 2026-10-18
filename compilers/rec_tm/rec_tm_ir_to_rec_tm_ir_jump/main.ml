open Utils_runtime.Machine
open Rec_tm_shared.Syntax
module I = Rec_tm_ir.Main
module J = Rec_tm_ir_jump.Main
module Var_id = Utils_names.Id.Make ()
module Label_id = Utils_names.Id.Make ()
module Var_names = Utils_names.Naming.Make (Var_id)
module Label_names = Utils_names.Naming.Make (Label_id)

let flatten_program p =
  let* () = I.validate p in
  Utils_base.Error.protect (fun () ->
      let variables = ref Var_names.empty and labels = ref Label_names.empty in
      let variable_supply = ref Var_id.Supply.empty in
      let label_supply = ref Label_id.Supply.empty in
      let variable hint =
        let id = Var_id.Supply.take variable_supply in
        let name, next = Var_names.assign ~hint id !variables in
        variables := next;
        name
      in
      let label hint =
        let id = Label_id.Supply.take label_supply in
        let name, next = Label_names.assign ~hint id !labels in
        labels := next;
        name
      in
      List.iter
        (fun (f : I.func) ->
          List.iter
            (fun name -> variables := Var_names.reserve name !variables)
            (I.collect_vars f.blocks);
          List.iter
            (fun (b : I.block) -> labels := Label_names.reserve b.label !labels)
            f.blocks)
        p.functions;
      let lower_blocks (blocks : I.block list) =
        let a = Array.of_list blocks in
        List.mapi
          (fun idx (b : I.block) ->
            {
              b with
              body =
                List.map
                  (function
                    | I.Continue c -> I.Jump (b.label, c)
                    | I.Break c ->
                        if idx + 1 = Array.length a then I.Return c
                        else I.Jump (a.(idx + 1).label, c)
                    | s -> s)
                  b.body;
            })
          blocks
      in
      let rec expand_blocks blocks = List.concat_map expand_block blocks
      and expand_block (block : I.block) =
        let rec loop prefix = function
          | [] -> [ { block with body = List.rev prefix } ]
          | I.Call name :: tail ->
              let callee = I.find_function p name in
              let vars = I.collect_vars callee.blocks in
              let variables =
                List.map (fun name -> (name, variable name)) vars
              in
              let labels =
                List.map
                  (fun (b : I.block) -> (b.label, label b.label))
                  callee.blocks
              in
              let cont = label (Utils_names.Identifier.make_exn "call_ret") in
              let init_label =
                label (Utils_names.Identifier.make_exn "call_init")
              in
              let rename name = List.assoc name labels in
              let var name =
                match List.assoc_opt name variables with
                | Some name -> name
                | None -> name
              in
              let lv : lvalue -> lvalue = function
                | Var s -> Var (var s)
                | Head -> Head
              in
              let rv : rvalue -> rvalue = function
                | Var s -> Var (var s)
                | Head -> Head
                | Const s -> Const s
              in
              let cond =
                Option.map (fun c -> { left = rv c.left; right = rv c.right })
              in
              let renamed =
                List.map
                  (fun (b : I.block) ->
                    {
                      I.label = rename b.label;
                      body =
                        List.map
                          (function
                            | I.Assign (dst, src) -> I.Assign (lv dst, rv src)
                            | I.Break c -> I.Break (cond c)
                            | I.Continue c -> I.Continue (cond c)
                            | I.Jump (label, c) -> I.Jump (rename label, cond c)
                            | I.Return c -> I.Jump (cont, cond c)
                            | s -> s)
                          b.body;
                    })
                  (lower_blocks callee.blocks)
              in
              let init =
                List.map
                  (fun name -> I.Assign (Var (var name), Const None))
                  vars
              in
              let renamed = { I.label = init_label; body = init } :: renamed in
              [ { block with body = List.rev prefix } ]
              @ expand_blocks renamed
              @ expand_block { I.label = cont; body = tail }
          | s :: tail -> loop (s :: prefix) tail
        in
        loop [] block.body
      in
      let main = I.find_function p (Utils_names.Identifier.make_exn "main") in
      {
        I.alphabet = p.alphabet;
        functions =
          [ { main with blocks = expand_blocks (lower_blocks main.blocks) } ];
      })

let compile p =
  let* flat = flatten_program p in
  Utils_base.Error.protect (fun () ->
      let main =
        I.find_function flat (Utils_names.Identifier.make_exn "main")
      in
      let blocks = Array.of_list main.blocks in
      let labels = Hashtbl.create 16 and offset = ref 0 in
      let stmts =
        List.concat
          (List.mapi
             (fun idx (b : I.block) ->
               if Hashtbl.mem labels b.label then
                 failwith
                   ("label '"
                   ^ Utils_names.Identifier.to_string b.label
                   ^ "' is duplicated");
               Hashtbl.add labels b.label !offset;
               offset := !offset + List.length b.body;
               List.map
                 (function
                   | I.Break c ->
                       if idx + 1 = Array.length blocks then I.Return c
                       else I.Jump (blocks.(idx + 1).label, c)
                   | I.Continue c -> I.Jump (b.label, c)
                   | s -> s)
                 b.body)
             main.blocks)
      in
      let body =
        List.map
          (function
            | I.Lt -> J.Lt
            | I.Rt -> J.Rt
            | I.Assign (dst, src) -> J.Assign (dst, src)
            | I.Jump (label, c) ->
                let target =
                  match Hashtbl.find_opt labels label with
                  | Some t -> t
                  | None ->
                      failwith
                        ("jump target '"
                        ^ Utils_names.Identifier.to_string label
                        ^ "' not found")
                in
                J.Jump (target, c)
            | I.Return c -> J.Jump (!offset, c)
            | _ -> failwith "Unflattened instruction")
          stmts
      in
      { J.alphabet = flat.alphabet; body })

module Source = I
module Target = J

let encode_ainput t = Ok t
let encode_rinput () = Ok ()
let decode_routput () = Ok ()
let decode_foutput t = Ok t
