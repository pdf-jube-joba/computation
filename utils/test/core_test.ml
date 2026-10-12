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
  let parsed = get (Environment.parse Number.of_string "z=1\n\na = 2\nz=3\n") in
  assert (parsed = [ ("z", "3"); ("a", "2") ]);
  let text = Environment.print ~trailing_newline:true Number.to_string parsed in
  assert (text = "z = 3\na = 2\n");
  assert (Environment.parse Number.of_string text = Ok parsed);
  assert (Environment.print ~trailing_newline:true Number.to_string [] = "");
  assert (Environment.parse (fun s -> Ok s) "x=a=b" = Ok [ ("x", "a=b") ]);
  assert (Environment.parse Number.of_string "x" = Error "Invalid env line: x");
  assert (
    Environment.parse Number.of_string "bad-name=1"
    = Error "Invalid identifier: bad-name");
  assert (
    Environment.parse Number.of_string "x=bad"
    = Error "Expected a natural number");
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
  assert (Number.of_string "00012" = Ok "12");
  let parser = Parser.create "/*a /*nested*/ comment*/ name // EOF comment" in
  assert (Parser.identifier parser = "name");
  assert (Parser.eof parser);
  print_endline "Utility tests passed"
