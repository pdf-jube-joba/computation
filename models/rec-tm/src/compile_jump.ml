open Utils.Machine
open Ir_common
module I = Rec_tm_ir
module J = Rec_tm_ir_jump
module L = Utils.Lexer

let flatten_program p =
  let* () = I.validate p in
  L.protect (fun () ->
      let counter = ref 0 in
      let reserved = Hashtbl.create 32 in
      List.iter
        (fun (f : I.func) ->
          List.iter
            (fun name -> Hashtbl.replace reserved name ())
            (I.collect_vars f.blocks);
          List.iter
            (fun (b : I.block) -> Hashtbl.replace reserved b.label ())
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
              let rec fresh () =
                let suffix = !counter in
                incr counter;
                let rename s = Printf.sprintf "__flat%d_%s" suffix s in
                let cont = Printf.sprintf "__cont_%s_%d" block.label suffix in
                let names =
                  cont
                  :: Printf.sprintf "__init%d" suffix
                  :: List.map rename vars
                  @ List.map (fun (b : I.block) -> rename b.label) callee.blocks
                in
                if List.exists (Hashtbl.mem reserved) names then fresh ()
                else (
                  List.iter (fun name -> Hashtbl.replace reserved name ()) names;
                  (suffix, cont, rename))
              in
              let suffix, cont, rename = fresh () in
              let var s = if List.mem s vars then rename s else s in
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
                  (fun name -> I.Assign (Var (rename name), Const None))
                  vars
              in
              let renamed =
                { I.label = Printf.sprintf "__init%d" suffix; body = init }
                :: renamed
              in
              [ { block with body = List.rev prefix } ]
              @ expand_blocks renamed
              @ expand_block { I.label = cont; body = tail }
          | s :: tail -> loop (s :: prefix) tail
        in
        loop [] block.body
      in
      let main = I.find_function p "main" in
      {
        I.alphabet = p.alphabet;
        functions =
          [ { main with blocks = expand_blocks (lower_blocks main.blocks) } ];
      })

let compile p =
  let* flat = flatten_program p in
  L.protect (fun () ->
      let main = I.find_function flat "main" in
      let blocks = Array.of_list main.blocks in
      let labels = Hashtbl.create 16 and offset = ref 0 in
      let stmts =
        List.concat
          (List.mapi
             (fun idx (b : I.block) ->
               if Hashtbl.mem labels b.label then
                 failwith ("label '" ^ b.label ^ "' is duplicated");
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
                  | None -> failwith ("jump target '" ^ label ^ "' not found")
                in
                J.Jump (target, c)
            | I.Return c -> J.Jump (!offset, c)
            | _ -> failwith "Unflattened instruction")
          stmts
      in
      { J.alphabet = flat.alphabet; body })

module Source = I.Machine
module Target = J.Machine

let encode_ainput t = Ok t
let encode_rinput () = Ok ()
let decode_routput () = Ok ()
let decode_foutput t = Ok t

let api : Utils.Compiler.api =
  {
    Utils.Compiler.identity_api with
    compile_code =
      (fun text ->
        let* p = I.parse_code text in
        let* p = compile p in
        Ok (J.print p));
  }
