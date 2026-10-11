open Utils

let get = function Ok x -> x | Error e -> failwith e

let () =
  let module G = Goto_lang in
  let code = get (G.Code.parse "// comment\ninc x; cpy y <- x; dec x;") in
  let state =
    get (G.make code (get (G.Ainput.parse "x=999999999999999999999999999999")))
  in
  let next =
    match get (G.step state ()) with
    | Machine.Continue { next; _ } -> next
    | _ -> assert false
  in
  assert (List.assoc "x" next.env = "1000000000000000000000000000000");
  let snapshot = G.snapshot next in
  assert (get (G.restore snapshot) = next);
  let next2 =
    match get (G.step next ()) with
    | Machine.Continue { next; _ } -> next
    | _ -> assert false
  in
  assert (List.assoc "y" next2.env = "1000000000000000000000000000000");
  print_endline "GOTO tests passed"
