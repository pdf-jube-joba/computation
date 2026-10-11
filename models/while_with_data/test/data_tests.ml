module A = While_with_data.Strarr
module T = While_with_data.Strtree

let ok = function Ok v -> v | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "Expected error"

let rec run_a fuel s =
  if fuel = 0 then failwith "Array machine did not halt"
  else
    match ok (A.step s ()) with
    | Utils.Machine.Halt env -> env
    | Continue { next; _ } -> run_a (fuel - 1) next

let rec run_t fuel s =
  if fuel = 0 then failwith "Tree machine did not halt"
  else
    match ok (T.step s ()) with
    | Utils.Machine.Halt env -> env
    | Continue { next; _ } -> run_t (fuel - 1) next

let next_a s =
  match ok (A.step s ()) with
  | Continue { next; output = () } -> next
  | _ -> failwith "Expected continue"

let next_t s =
  match ok (T.step s ()) with
  | Continue { next; output = () } -> next
  | _ -> failwith "Expected continue"

let () =
  let (_ : (module Utils.Machine.S)) = (module A.Machine) in
  let (_ : (module Utils.Machine.S)) = (module T.Machine) in
  assert (
    ok
      (A.Code.parse
         "/* outer /* inner */ */ x := ['/', '*']; // tail\ny := tail x")
    = A.Seq
        ( A.Assign ("x", A.Const [ "/"; "*" ]),
          A.Assign ("y", A.Tail (A.Var "x")) ));
  assert (
    ok
      (T.Code.parse "x := atom '/'; /* comment */ y := cons x atom '*' // tail")
    = T.Seq
        ( T.Assign ("x", T.Atom (T.AtomChar "/")),
          T.Assign ("y", T.Cons (T.Var "x", T.Atom (T.AtomChar "*"))) ));
  let text = "x := ['a', 'b'] ; while x == ['a', 'b'] { x := tail x }" in
  let code = ok (A.Code.parse text) in
  assert (ok (A.Code.parse (A.Code.print code)) = code);
  let env = run_a 40 (ok (A.make code [])) in
  assert (List.assoc "x" env = [ "b" ]);
  let code =
    ok
      (A.Code.parse
         "x := ['a'] ; y := head x ++ ['b'] ++ tail x ; if is_empty missing \
          then z := tail [] end")
  in
  let env = run_a 40 (ok (A.make code [])) in
  assert (List.assoc "y" env = [ "a"; "b" ]);
  assert (List.assoc "z" env = []);
  let env =
    ok
      (A.Ainput.parse
         "foo_bar = ['猫', '\\n', '\\r', '\\t', '\\\\', '\\'']\n\
          x = []\n\
          foo_bar = ['🙂']")
  in
  assert (List.assoc "foo_bar" env = [ "🙂" ]);
  assert (ok (A.Ainput.parse (A.Foutput.print env)) = env);
  let s =
    ok
      (A.make
         (A.Seq (A.Assign ("x", A.Const [ "a" ]), A.Assign ("y", A.Var "x")))
         [])
  in
  let original = A.snapshot s in
  let s1 = next_a s in
  assert (s1.stmt = A.Seq (A.Nop, A.Assign ("y", A.Var "x")));
  assert (A.snapshot s = original);
  let s2 = next_a s1 in
  assert (s2.stmt = A.Assign ("y", A.Var "x"));
  let restored =
    ok
      (A.restore
         (Yojson.Safe.from_string (Yojson.Safe.to_string (A.snapshot s2))))
  in
  assert (A.snapshot (next_a s2) = A.snapshot (next_a restored));
  let loop =
    ok
      (A.make
         (A.While (A.IsEmpty (A.Var "x"), A.Assign ("x", A.Const [ "a" ])))
         [])
  in
  let loop = next_a loop in
  assert (
    loop.stmt
    = A.If
        ( A.IsEmpty (A.Var "x"),
          A.Seq
            ( A.Assign ("x", A.Const [ "a" ]),
              A.While (A.IsEmpty (A.Var "x"), A.Assign ("x", A.Const [ "a" ]))
            ) ));
  let code =
    ok (T.Code.parse "x := atom 'a' ; ifeq is_atom x then y := cons x x end")
  in
  assert (ok (T.Code.parse (T.Code.print code)) = code);
  let code =
    ok
      (T.Code.parse
         "x := cons atom 'a' atom 'b' ; y := left x ; z := atom mtoa right x ; \
          ifeq y == atom 'a' then k := right x end")
  in
  assert (ok (T.Code.parse (T.Code.print code)) = code);
  let env = run_t 60 (ok (T.make code [])) in
  assert (List.assoc "y" env = T.AtomValue "a");
  assert (List.assoc "z" env = T.AtomValue "b");
  assert (List.assoc "k" env = T.AtomValue "b");
  let code = ok (T.Code.parse "while is_atom x { x := cons x atom 'z' }") in
  let env = run_t 30 (ok (T.make code [ ("x", T.AtomValue "a") ])) in
  assert (List.assoc "x" env = T.ConsValue (T.AtomValue "a", T.AtomValue "z"));
  let env =
    ok
      (T.Ainput.parse
         "x = cons atom '猫' cons atom '\\n' atom '🙂'\n\
          x = atom '\\''\n\
          foo_bar = atom 'é'")
  in
  assert (ok (T.Ainput.parse (T.Foutput.print env)) = env);
  let s =
    ok
      (T.make
         (ok (T.Code.parse "y := left x"))
         [ ("x", T.ConsValue (T.AtomValue "a", T.AtomValue "b")) ])
  in
  let restored =
    ok
      (T.restore
         (Yojson.Safe.from_string (Yojson.Safe.to_string (T.snapshot s))))
  in
  assert (T.snapshot (next_t s) = T.snapshot (next_t restored));
  List.iter
    (fun text -> error (T.step (ok (T.make (ok (T.Code.parse text)) [])) ()))
    [
      "x := y";
      "x := left atom 'a'";
      "x := right atom 'a'";
      "x := atom mtoa cons atom 'a' atom 'b'";
    ];
  List.iter
    (fun text -> error (T.Ainput.parse text))
    [ "x = y"; "x = atom mtoa atom 'a'"; "x = left cons atom 'a' atom 'b'" ];
  List.iter
    (fun text -> error (A.Code.parse text))
    [
      "";
      "x := ['ab']";
      "x := [\"a\"]";
      "x := [] /* unclosed";
      "x := ['a',]";
      "x := [] ;";
      "if is_empty x then Nop";
      "x := [] junk";
    ];
  List.iter
    (fun text -> error (T.Code.parse text))
    [
      "";
      "x := atom ''";
      "x := atom 'ab'";
      "x := atom \"a\"";
      "x := atom 'a' /* unclosed";
      "while is_atom x { Nop";
      "x := cons atom 'a'";
    ];
  error (A.Rinput.parse "x");
  error (T.Rinput.parse "x");
  assert (A.Routput.print () = "");
  assert (T.Routput.print () = "");
  assert (match A.render s1 with `List [ _; _ ] -> true | _ -> false);
  assert (match T.render s with `List [ _; _ ] -> true | _ -> false);
  error (A.Code.parse "x := Nop");
  error (T.Code.parse "x := then");
  let bad =
    ok
      (T.make
         (ok (T.Code.parse "ifeq left atom 'a' == right atom 'b' then Nop end"))
         [])
  in
  assert (T.step bad () = Error "left expects a cons cell");
  print_endline "strarr and strtree tests passed"
