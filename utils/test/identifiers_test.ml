open Utils

let parse_identifier text =
  Lexer.protect (fun () ->
      let parser = Lexer.create text in
      let name = Lexer.identifier parser in
      Lexer.expect_eof parser;
      name)

let parsers =
  [
    ("identifier", Identifier.make);
    ("JSON identifier", fun name -> Identifier.of_yojson (`String name));
    ("lexer identifier", parse_identifier);
  ]

let () =
  let valid = [ "foo_bar"; "_"; "_temp_1"; "A0_Z" ] in
  let invalid = [ ""; "foo-bar"; "foo'"; "0foo"; "123"; "foo.bar"; "猫" ] in
  List.iter (fun name -> assert (Identifier.valid name)) valid;
  List.iter (fun name -> assert (not (Identifier.valid name))) invalid;
  List.iter
    (fun (context, parse) ->
      List.iter
        (fun name ->
          if parse name <> Ok name then
            failwith (context ^ ": rejected valid identifier " ^ name))
        valid;
      List.iter
        (fun name ->
          if Result.is_ok (parse name) then
            failwith (context ^ ": accepted invalid identifier " ^ name))
        invalid)
    parsers;
  List.iter
    (fun json -> assert (Result.is_error (Identifier.of_yojson json)))
    [ `Null; `Int 1; `Bool false; `List []; `Assoc [] ];
  print_endline "Identifier and lexer tests passed"
