module T = Turing_machine

let get = function Ok v -> v | Error e -> failwith e
let tape = T.Tape.parse
let check s b = if not b then failwith s

let run s =
  let rec loop n s =
    if n = 0 then failwith "step limit"
    else
      match get (T.step s ()) with
      | Utils.Machine.Halt t -> t
      | Utils.Machine.Continue { next; _ } -> loop (n - 1) next
  in
  loop 100 s

let () =
  List.iter
    (fun text -> check "tape roundtrip" (T.Tape.print (get (tape text)) = text))
    [ "-|d|e,f,g"; "a|b|c"; "-|-|-"; "a,b|-|c,d"; "|-|" ];
  let v =
    List.map
      (fun s -> get (T.Sign.parse s))
      [ "a"; "b"; "c"; "d"; "e"; "f"; "g" ]
  in
  let t = get (T.tape_from_vec v 3) in
  check "move left"
    (T.tape_equal (T.move_to t T.Left) (get (T.tape_from_vec v 2)));
  check "left-right" (T.tape_equal (T.move_to (T.move_to t T.Left) T.Right) t);
  let t = get (T.tape_from_vec v 1) in
  let t = List.fold_left (fun t _ -> T.move_to t T.Left) t [ (); (); () ] in
  check "infinite blanks"
    (T.tape_equal t (get (T.tape_from_vec (None :: None :: v) 0)));
  check "ignore outer blanks"
    (T.tape_equal (get (tape "-|-|-")) (get (tape "|-|")));
  check "from_vec end"
    (T.tape_equal (get (T.tape_from_vec v 7)) (get (tape "a,b,c,d,e,f,g|-|")));
  List.iter
    (fun s -> check "invalid tape" (Result.is_error (tape s)))
    [ ""; "a|b"; "a||b"; "|0|"; "|a|b," ];
  let code =
    "start\ngoal\na,start,b,start,R\nb,start,b,start,R\nc,start,c,goal,C\n"
  in
  let d = get (T.Code.parse code) in
  check "code roundtrip" (get (T.Code.parse (T.Code.print d)) = d);
  let s = get (T.make d (get (tape "-|a|b,a,b,b,c"))) in
  let restored = get (T.restore (T.snapshot s)) in
  check "snapshot" (T.snapshot restored = T.snapshot s);
  check "termination" (T.Tape.print (run restored) = "-,b,b,b,b,b|c|");
  check "missing transition halts"
    (T.tape_equal (run (get (T.make d (get (tape "|-|"))))) (get (tape "|-|")));
  List.iter
    (fun s -> check "invalid code" (Result.is_error (T.Code.parse s)))
    [
      "start\ngoal\na,start,a,start,C\na,start,a,goal,R";
      "start\ngoal\na,goal,a,start,C";
      "start\ngoal\na,start,a,goal,X";
    ];
  let module B = T.Manipulation.Builder in
  let module G = T.Manipulation.Graph_compose in
  let first =
    {
      B.name = "first";
      init_state = "s";
      accepted_state = [ "done" ];
      code = [ B.of_entry ((None, "s"), (Some "a", "done", T.Right)) ];
    }
  in
  let second =
    {
      B.name = "second";
      init_state = "s";
      accepted_state = [ "done" ];
      code = [ B.of_entry ((None, "s"), (Some "a", "done", T.Constant)) ];
    }
  in
  let graph =
    {
      G.name = "joined";
      init_state = "start";
      assign_vertex_to_builder = [ first; second ];
      assign_edge_to_state = [ ((0, 1), "done") ];
      acceptable = [ []; [ "done" ] ];
    }
  in
  let builder = get (G.builder_composition graph) in
  let s = get (B.build ~sign:Fun.id builder (get (tape "|-|"))) in
  check "graph composition" (T.tape_equal (run s) (get (tape "a|a|")));
  check "graph validation"
    (Result.is_error (G.builder_composition { graph with acceptable = [] }));
  print_endline
    "Turing tape, parser, machine, snapshot and composition tests passed"
