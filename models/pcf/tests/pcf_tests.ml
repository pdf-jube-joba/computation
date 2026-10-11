open Pcf
module Checked_expr : Utils.Machine.S = Expr_machine
module Checked_stmt : Utils.Machine.S = Expr_stmt_machine
module Checked_cek : Utils.Machine.S = Cek_machine
module Checked_secd : Utils.Machine.S = Secd_machine
module Checked_cek_compiler : Utils.Compiler.S = Compiler.Expr_to_cek
module Checked_secd_compiler : Utils.Compiler.S = Compiler.Expr_to_secd

let get = function Ok x -> x | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "expected error"

let run (type c a s r f)
    (module M : Utils.Machine.S
      with type code = c
       and type ainput = a
       and type state = s
       and type rinput = unit
       and type routput = r
       and type foutput = f) code input =
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
    | Utils.Machine.Halt n -> (List.rev outputs, M.foutput_json n)
    | Utils.Machine.Continue { next; output } ->
        let outputs =
          match M.routput_json output with
          | `String "" -> outputs
          | out -> out :: outputs
        in
        loop (fuel - 1) next outputs
  in
  loop 2000 (get (M.make code input)) []

let differential text inputs expected =
  let source = get (Expr_machine.parse_code text) in
  let inputs = get (Syntax.parse_ainput inputs) in
  let expr = run (module Expr_machine) source inputs in
  assert (expr = expected);
  let cek = get (Compiler.Expr_to_cek.compile source) in
  assert (run (module Cek_machine) cek inputs = expr);
  let secd = get (Compiler.Expr_to_secd.compile source) in
  assert (run (module Secd_machine) secd inputs = expr);
  let compiled = get (Compiler.expr_to_secd.compile_code text) in
  assert (
    run (module Secd_machine) (get (Secd_machine.parse_code compiled)) inputs
    = expr);
  assert (get (Secd_machine.parse_code (Secd_machine.print_code secd)) = secd);
  assert (get (Syntax.parse_code (Syntax.print source)) = source)

let stmt text expected =
  let source = get (Expr_stmt_machine.parse_code text) in
  assert (run (module Expr_stmt_machine) source [] = expected);
  assert (get (Expr_stmt_machine.parse_code (Syntax.print source)) = source)

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
      let e = get (Syntax.parse_code text) in
      let test
          (module M : Utils.Machine.S
            with type code = Syntax.expr
             and type ainput = Utils.Number.t list
             and type rinput = unit) =
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
      test (module Expr_machine);
      test (module Cek_machine);
      let state = get (Secd_machine.make (Secd_machine.compile_expr e) []) in
      let rec loop fuel state =
        if fuel = 0 then failwith "expected SECD error"
        else
          match Secd_machine.step state () with
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
  error (Syntax.parse_code "fun if => 1");
  error (Syntax.parse_code "1 trailing");
  error (Syntax.parse_stmt_code "print 1");
  error (Syntax.parse_code "{ skip; 1 }");
  error (Syntax.parse_stmt_code "{ 1 }");
  error (Syntax.parse_stmt_code "{ skip; }");
  assert (get (Syntax.parse_ainput "(,001,,2,)") = [ "1"; "2" ]);
  assert (get (Syntax.parse_ainput "(+001, +2)") = [ "1"; "2" ]);
  assert (get (Syntax.parse_effect "+001") = Some "1");
  assert (get (Compiler.expr_to_secd.encode_ainput "(+001)") = "(1)");
  assert (get (Compiler.expr_to_cek.decode_routput "+001") = "1");
  assert (get (Compiler.expr_to_secd.decode_foutput "+001") = "1");
  error (Syntax.parse_ainput "(+ 1)");
  error (Syntax.parse_effect "+");
  error (Syntax.parse_effect "++1");
  error (Syntax.parse_ainput "1,2");
  error (Expr_machine.parse_rinput "1");
  let e = get (Syntax.parse_code "fun y => x") in
  let substituted = Syntax.substitute e "x" (Syntax.Var "y") in
  assert (Syntax.Names.elements (Syntax.free_vars substituted) = [ "y" ]);
  assert (substituted = Syntax.Fun ("y_0", Syntax.Var "y"));
  (* A generated binder must not become the substitution target, even when
     the target did not originally occur. Rust also missed this collision. *)
  let absent = Syntax.Fun ("y", Syntax.Var "y") in
  let substituted = Syntax.substitute absent "y_0" (Syntax.Var "y") in
  assert (Syntax.Names.is_empty (Syntax.free_vars substituted));
  assert (substituted = Syntax.Fun ("y_1", Syntax.Var "y_1"));
  let absent =
    Syntax.Rec ("f", "y", Syntax.App (Syntax.Var "f", Syntax.Var "y"))
  in
  let substituted = Syntax.substitute absent "f_0" (Syntax.Var "f") in
  assert (Syntax.Names.is_empty (Syntax.free_vars substituted));
  assert (
    substituted
    = Syntax.Rec ("f_1", "y", Syntax.App (Syntax.Var "f_1", Syntax.Var "y")));
  assert (
    get
      (Secd_machine.parse_code {|[{"PushNat":2},{"PushNat":3},{"BinOp":"Add"}]|})
    = [
        Secd_machine.PushNat "2";
        Secd_machine.PushNat "3";
        Secd_machine.BinOp Syntax.Add;
      ]);
  assert (
    run
      (module Secd_machine)
      (get
         (Secd_machine.parse_code
            {|[{"PushNat":2},{"PushNat":3},{"BinOp":"Add"}]|}))
      []
    = ([], s "5"));
  error (Secd_machine.parse_code {|[{"Acc":"bad!"}]|});
  error (Secd_machine.step (get (Secd_machine.make [ Secd_machine.Ret ] [])) ());
  error (Secd_machine.step (get (Secd_machine.make [ Secd_machine.Ap ] [])) ());
  assert (Expr_machine.routput_json None = s "");
  print_endline "PCF semantic/compiler tests passed"
