let delimited_list ~opening ~closing ~separator parse_item text =
  Parser.protect (fun () ->
      let text = String.trim text in
      let n = String.length text in
      if n < 2 || text.[0] <> opening || text.[n - 1] <> closing then
        failwith
          (Printf.sprintf "Expected to start with '%c' and end with '%c'"
             opening closing);
      let items =
        String_ext.split_nonempty separator (String.sub text 1 (n - 2))
      in
      List.map
        (fun item ->
          match parse_item item with
          | Ok value -> value
          | Error error -> failwith error)
        items)
