open Utils

let () =
  let valid = [ "foo_bar"; "_"; "_temp_1"; "A0_Z" ] in
  let invalid = [ ""; "foo-bar"; "foo'"; "0foo"; "123"; "foo.bar"; "猫" ] in
  List.iter (fun name -> assert (Identifier.valid name)) valid;
  List.iter (fun name -> assert (not (Identifier.valid name))) invalid;
  List.iter
    (fun json -> assert (Result.is_error (Identifier.of_yojson json)))
    [ `Null; `Int 1; `Bool false; `List []; `Assoc [] ];
  print_endline "Identifier tests passed"
