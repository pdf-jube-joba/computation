module Checked_expr : Utils.Machine.S = Pcf_expr.Main
module Checked_stmt : Utils.Machine.S = Pcf_expr_stmt.Main
module Checked_cek : Utils.Machine.S = Pcf_cek.Main
module Checked_secd : Utils.Machine.S = Pcf_secd.Main
module Checked_cek_compiler : Utils.Compiler.S = Pcf_expr_to_pcf_cek.Main
module Checked_secd_compiler : Utils.Compiler.S = Pcf_expr_to_pcf_secd.Main

let get = function Ok x -> x | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "expected error"

let run (type c a s r f)
    (module M : Utils.Machine.S
      with type Code.t = c
       and type Ainput.t = a
       and type state = s
       and type Rinput.t = unit
       and type Routput.t = r
       and type Foutput.t = f) code input =
  let rec loop fuel state outputs =
    if fuel = 0 then failwith "machine did not halt";
    (* Check resumability at every boundary, including closures and continuations. *)
    let state =
      get
        (M.restore
           (Yojson.Safe.from_string (Yojson.Safe.to_string (M.snapshot state))))
    in
    assert (match M.render state with `List _ -> true | _ -> false);
    match get (M.step state ()) with
    | Utils.Machine.Halt n -> (List.rev outputs, `String (M.Foutput.print n))
    | Utils.Machine.Continue { next; output } ->
        let outputs =
          match M.Routput.print output with
          | "" -> outputs
          | out -> `String out :: outputs
        in
        loop (fuel - 1) next outputs
  in
  loop 2000 (get (M.make code input)) []

let differential text inputs expected =
  let source = get (Pcf_expr.Main.Code.parse text) in
  let inputs = get (Pcf_shared.Syntax.Ainput.parse inputs) in
  let expr = run (module Pcf_expr.Main) source inputs in
  assert (expr = expected);
  let cek = get (Pcf_expr_to_pcf_cek.Main.compile source) in
  assert (run (module Pcf_cek.Main) cek inputs = expr);
  let secd = get (Pcf_expr_to_pcf_secd.Main.compile source) in
  assert (run (module Pcf_secd.Main) secd inputs = expr);
  let compiled = get (Pcf_expr_to_pcf_secd.Main.api.compile_code text) in
  assert (
    run (module Pcf_secd.Main) (get (Pcf_secd.Main.Code.parse compiled)) inputs
    = expr);
  assert (get (Pcf_secd.Main.Code.parse (Pcf_secd.Main.Code.print secd)) = secd);
  assert (
    get (Pcf_shared.Syntax.Code.parse (Pcf_shared.Syntax.Code.print source))
    = source)

let stmt text expected =
  let source = get (Pcf_expr_stmt.Main.Code.parse text) in
  assert (run (module Pcf_expr_stmt.Main) source [] = expected);
  assert (
    get (Pcf_expr_stmt.Main.Code.parse (Pcf_shared.Syntax.Code.print source))
    = source)

let () =
  let s n = `String n in
  differential "(fun x => (fun y => 7))(print 1)(print 2)" "()"
    ([ s "1"; s "2" ], s "7");
  differential "rec f x => if #true then x else f(x) fi" "(5)" ([], s "5");
  differential "fun x => fun y => inc (x - y)" "(10, 3)" ([], s "8");
  differential "(fun x => (fun f => f(3))(fun y => x + y))(4)" "()" ([], s "7");
  differential
    "(rec f x => fun b => if b then x else f(inc x)(#true) fi)(4)(#false)" "()"
    ([], s "5");
  differential "(fun x => (fun y => 0)(x))(print 42)" "()" ([ s "42" ], s "0");
  differential "if not (#false && #true) then 8 else print 99 fi" "()"
    ([], s "8");
  differential "if #false && (fun z => #true)(print 6) then print 99 else 4 fi"
    "()"
    ([ s "6" ], s "4");
  differential "dec inc 0 + 8 - 3" "()" ([], s "5");
  differential "fun x => x" "(+005)" ([], s "5");
  differential
    "(fun x => (fun f => (fun x => f(2))(9))(rec f y => if #true then x + y \
     else f(y) fi))(4)"
    "()"
    ([], s "6");
  (* Unlike dec in the continuation language, PCF dec and subtraction reject underflow. *)
  List.iter
    (fun text ->
      let e = get (Pcf_shared.Syntax.Code.parse text) in
      let test
          (module M : Utils.Machine.S
            with type Code.t = Pcf_shared.Syntax.expr
             and type Ainput.t = Utils.Number.t list
             and type Rinput.t = unit) =
        let rec loop fuel state =
          if fuel = 0 then failwith "expected failure"
          else
            match M.step state () with
            | Error _ -> ()
            | Ok (Utils.Machine.Halt _) -> failwith "expected failure"
            | Ok (Utils.Machine.Continue { next; _ }) -> loop (fuel - 1) next
        in
        loop 100 (get (M.make e []))
      in
      test (module Pcf_expr.Main);
      test (module Pcf_cek.Main);
      let state = get (Pcf_secd.Main.make (Pcf_secd.Main.compile_expr e) []) in
      let rec loop fuel state =
        if fuel = 0 then failwith "expected SECD error"
        else
          match Pcf_secd.Main.step state () with
          | Error _ -> ()
          | Ok (Utils.Machine.Halt _) -> failwith "expected SECD error"
          | Ok (Utils.Machine.Continue { next; _ }) -> loop (fuel - 1) next
      in
      loop 100 state)
    [
      "dec 0";
      "1 - 2";
      "#true + 1";
      "if 0 then 1 else 2 fi";
      "0(1)";
      "x";
      "#unit";
      "#false && (print 9)";
    ];
  stmt "{ print 1; 2 } + { print 3; 4 }" ([ s "1"; s "3" ], s "6");
  stmt "({ let x := 1; fun y => x + y })(2)" ([], s "3");
  stmt "{ let x := 1; let x := 4; x }" ([], s "4");
  stmt "{ let x := 1; let y := { let x := 2; x }; x + y }" ([], s "3");
  stmt "{ if #true then print 1; let x := 2 end; x }" ([ s "1" ], s "2");
  stmt "{ let b := #true; while b do print 9; let b := #false end; 7 }"
    ([ s "9" ], s "7");
  stmt "{ while #false do print 99 end; if #false then print 98 end; 6 }"
    ([], s "6");
  stmt "(rec f x => { skip; if #true then x else f(x) fi })(5)" ([], s "5");
  stmt
    "{ let x := 4; let f := rec f y => if #true then x + y else f(y) fi; let x \
     := 9; f(2) }"
    ([], s "6");
  error (Pcf_shared.Syntax.Code.parse "fun if => 1");
  error (Pcf_shared.Syntax.Code.parse "1 trailing");
  error (Pcf_shared.Syntax.Stmt_code.parse "print 1");
  error (Pcf_shared.Syntax.Code.parse "{ skip; 1 }");
  error (Pcf_shared.Syntax.Stmt_code.parse "{ 1 }");
  error (Pcf_shared.Syntax.Stmt_code.parse "{ skip; }");
  assert (get (Pcf_shared.Syntax.Ainput.parse "(,001,,2,)") = [ "1"; "2" ]);
  assert (get (Pcf_shared.Syntax.Ainput.parse "(+001, +2)") = [ "1"; "2" ]);
  assert (get (Pcf_shared.Syntax.Routput.parse "+001") = Some "1");
  assert (get (Pcf_expr_to_pcf_secd.Main.api.encode_ainput "(+001)") = "(1)");
  assert (get (Pcf_expr_to_pcf_cek.Main.api.decode_routput "+001") = "1");
  assert (get (Pcf_expr_to_pcf_secd.Main.api.decode_foutput "+001") = "1");
  error (Pcf_shared.Syntax.Ainput.parse "(+ 1)");
  error (Pcf_shared.Syntax.Routput.parse "+");
  error (Pcf_shared.Syntax.Routput.parse "++1");
  error (Pcf_shared.Syntax.Ainput.parse "1,2");
  error (Pcf_expr.Main.Rinput.parse "1");
  let e = get (Pcf_shared.Syntax.Code.parse "fun y => x") in
  let substituted =
    Pcf_shared.Syntax.substitute e "x" (Pcf_shared.Syntax.Var "y")
  in
  assert (
    Pcf_shared.Syntax.Names.elements (Pcf_shared.Syntax.free_vars substituted)
    = [ "y" ]);
  assert (substituted = Pcf_shared.Syntax.Fun ("y_0", Pcf_shared.Syntax.Var "y"));
  (* A generated binder must not become the substitution target, even when
     the target did not originally occur. Rust also missed this collision. *)
  let absent = Pcf_shared.Syntax.Fun ("y", Pcf_shared.Syntax.Var "y") in
  let substituted =
    Pcf_shared.Syntax.substitute absent "y_0" (Pcf_shared.Syntax.Var "y")
  in
  assert (
    Pcf_shared.Syntax.Names.is_empty (Pcf_shared.Syntax.free_vars substituted));
  assert (
    substituted = Pcf_shared.Syntax.Fun ("y_1", Pcf_shared.Syntax.Var "y_1"));
  let absent =
    Pcf_shared.Syntax.Rec
      ( "f",
        "y",
        Pcf_shared.Syntax.App
          (Pcf_shared.Syntax.Var "f", Pcf_shared.Syntax.Var "y") )
  in
  let substituted =
    Pcf_shared.Syntax.substitute absent "f_0" (Pcf_shared.Syntax.Var "f")
  in
  assert (
    Pcf_shared.Syntax.Names.is_empty (Pcf_shared.Syntax.free_vars substituted));
  assert (
    substituted
    = Pcf_shared.Syntax.Rec
        ( "f_1",
          "y",
          Pcf_shared.Syntax.App
            (Pcf_shared.Syntax.Var "f_1", Pcf_shared.Syntax.Var "y") ));
  assert (
    get
      (Pcf_secd.Main.Code.parse
         {|[{"PushNat":2},{"PushNat":3},{"BinOp":"Add"}]|})
    = [
        Pcf_secd.Main.PushNat "2";
        Pcf_secd.Main.PushNat "3";
        Pcf_secd.Main.BinOp Pcf_shared.Syntax.Add;
      ]);
  assert (
    run
      (module Pcf_secd.Main)
      (get
         (Pcf_secd.Main.Code.parse
            {|[{"PushNat":2},{"PushNat":3},{"BinOp":"Add"}]|}))
      []
    = ([], s "5"));
  error (Pcf_secd.Main.Code.parse {|[{"Acc":"bad!"}]|});
  error
    (Pcf_secd.Main.step (get (Pcf_secd.Main.make [ Pcf_secd.Main.Ret ] [])) ());
  error
    (Pcf_secd.Main.step (get (Pcf_secd.Main.make [ Pcf_secd.Main.Ap ] [])) ());
  assert (Pcf_expr.Main.Routput.print None = "");
  print_endline "PCF semantic/compiler tests passed"
