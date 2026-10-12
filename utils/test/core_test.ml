open Utils

let get = function Ok x -> x | Error e -> failwith e

let () =
  let env = [ ("z", 1); ("a", 2); ("z", 3) ] in
  assert (Assoc.set "z" 4 env = [ ("z", 4); ("a", 2); ("z", 4) ]);
  assert (Assoc.set "b" 5 env = env @ [ ("b", 5) ]);
  assert (Assoc.set_sorted "z" 4 env = [ ("a", 2); ("z", 4); ("z", 3) ]);
  assert (Assoc.lookup "a" env "missing" = 2);
  assert (
    Parser.protect (fun () -> Assoc.lookup "b" env "missing") = Error "missing");
  assert (List_ext.nth [ 1; 2 ] 1 "bounds" = 2);
  assert (List_ext.replace [ 1; 2 ] 1 3 "bounds" = [ 1; 3 ]);
  assert (
    Parser.protect (fun () -> List_ext.replace [ 1; 2 ] 2 3 "bounds")
    = Error "bounds");
  let used = Identifier.Set.of_list [ "_0"; "_2"; "x_0"; "x_1" ] in
  assert (Identifier.fresh used = "_1");
  assert (Identifier.fresh ~prefix:"x_" used = "x_2");
  assert (Identifier.valid (Identifier.fresh used));
  let check names =
    Parser.protect (fun () ->
        Identifier.check_unique ~message:(fun n -> "duplicate: " ^ n) names)
  in
  assert (check [ "x"; "y" ] = Ok ());
  assert (check [ "x"; "y"; "y"; "x" ] = Error "duplicate: y")

let () =
  assert (
    Number.add "999999999999999999999999999999" "1"
    = "1000000000000000000000000000000");
  assert (
    Number.sub "1000000000000000000000000000000" "1"
    = "999999999999999999999999999999");
  assert (Number.sub "2" "7" = "0");
  assert (
    Number.mul "12345678901234567890" "987654321"
    = "12193263112482853211126352690");
  assert (
    get (Number.divmod "12193263112482853211126352690" "987654321")
    = ("12345678901234567890", "0"));
  List.iter
    (fun c ->
      assert (Utf8.is_scalar c);
      assert (Utf8.scalar_length c 0 = String.length c);
      assert (Utf8.scalar ("x" ^ c ^ "y") 1 = (c, 1 + String.length c)))
    [ "a"; "é"; "あ"; "😀" ];
  assert (Utf8.valid "");
  assert (Utf8.valid "aéあ😀");
  assert (not (Utf8.is_scalar ""));
  assert (not (Utf8.is_scalar "ab"));
  List.iter
    (fun c ->
      assert (not (Utf8.valid c));
      assert (Result.is_error (Parser.protect (fun () -> Utf8.scalar c 0))))
    [ "\128"; "\192\128"; "\237\160\128"; "\244\144\128\128"; "\226\128" ];
  assert (String_ext.words "a\t b\r\nc" = [ "a"; "b"; "c" ]);
  assert (String_ext.lines " \na\n\tb\n" = [ "a"; "b" ]);
  assert (String_ext.split_once ":=" "x:=a:=b" = Some ("x", "a:=b"));
  assert (String_ext.split_once ":=" "x" = None);
  assert (String_ext.strip_prefix "@" "@name" = "name");
  assert (String_ext.strip_suffix ";" "name;" = "name");
  assert (String_ext.strip_line_comment "//" "a // comment" = "a ");
  List.iter
    (fun text -> assert (Number.to_bytes (Number.of_bytes text) = text))
    [ ""; "Hi"; "あ"; "\000\001"; "\255" ];
  assert (Number.to_bytes "256" = "\000\001");
  print_endline "Utility tests passed"
