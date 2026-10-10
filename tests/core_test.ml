open Utils
module _ : Utils.Compiler.S = Example_counter.Compiler

let get = function Ok x -> x | Error e -> failwith e

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
  let parser = Lexer.create "/*a /*nested*/ comment*/ name // EOF comment" in
  assert (Lexer.identifier parser = "name");
  assert (Lexer.eof parser);
  let module G = Goto_lang in
  let code = get (G.parse_code "// comment\ninc x; cpy y <- x; dec x;") in
  let state =
    get (G.make code (get (G.parse_ainput "x=999999999999999999999999999999")))
  in
  let next =
    match get (G.step state ()) with
    | Machine.Continue { next; _ } -> next
    | _ -> assert false
  in
  assert (G.get next.env "x" = "1000000000000000000000000000000");
  let snapshot = G.snapshot next in
  assert (get (G.restore snapshot) = next);
  let next2 =
    match get (G.step next ()) with
    | Machine.Continue { next; _ } -> next
    | _ -> assert false
  in
  assert (G.get next2.env "y" = "1000000000000000000000000000000");
  let module R = Recursive_function in
  let addition =
    get (R.parse_code "let add = PRIM[z:PROJ[1,0] s:COMP[SUCC:PROJ[3,0]]]. add")
  in
  assert (get (R.evaluate (get (R.make addition [ "3"; "5" ])).process) = "8");
  assert (R.parse_code (R.print addition) = Ok addition);
  let mu = get (R.parse_code "MUOP[PROJ[1,0]]") in
  assert (get (R.evaluate (get (R.make mu [])).process) = "0");
  let p = get (R.make addition [ "2"; "3" ]) in
  assert (get (R.restore (R.snapshot p)) = p);
  assert (match R.parse_code "PROJ[1,2]" with Error _ -> true | _ -> false);
  assert (
    match R.parse_code "PRIM[z:ZERO s:SUCC]" with Error _ -> true | _ -> false);
  print_endline
    "Core arithmetic, lexer, GOTO and recursive function tests passed"
