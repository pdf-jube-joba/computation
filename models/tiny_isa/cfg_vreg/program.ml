open Types

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
