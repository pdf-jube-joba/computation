open Utils.Machine
open Recursive_function
module C = Recursive_function_goto_lang
module Typed_contract : Utils.Compiler.S = C

let ok = function Ok value -> value | Error message -> failwith message
let error = function Error _ -> () | Ok _ -> failwith "Expected an error"
let numbers = List.map Utils.Number.of_int
let add = Prim (Proj (1, 0), Comp (Succ, [ Proj (3, 0) ]))
let predecessor = Prim (Zero, Proj (2, 1))
let subtract = Prim (Proj (1, 0), Comp (predecessor, [ Proj (3, 0) ]))
let zero_unary = Prim (Zero, Proj (2, 0))
let multiply = Prim (zero_unary, Comp (add, [ Proj (3, 0); Proj (3, 2) ]))

let run_code code env =
  let initial = ok (Goto_lang.make code env) in
  let rec run fuel state =
    if fuel = 0 then failwith "Goto evaluation step limit exceeded";
    match ok (Goto_lang.step state ()) with
    | Halt output -> output
    | Continue { next; _ } -> run (fuel - 1) next
  in
  run 100000 initial

let run_source initial =
  let rec loop fuel state =
    if fuel = 0 then failwith "Recursive function step limit exceeded";
    match ok (Recursive_function.step state ()) with
    | Halt output -> output
    | Continue { next; _ } -> loop (fuel - 1) next
  in
  loop 100000 initial

let check func input =
  let source = ok (Recursive_function.make func input) in
  let expected = run_source source in
  let code = ok (C.compile func) in
  assert (ok (Goto_lang.Code.parse (Goto_lang.Code.print code)) = code);
  let env = run_code code (ok (C.encode_ainput input)) in
  let actual = ok (C.decode_foutput env) in
  if not (Utils.Number.equal expected actual) then
    failwith
      (Code.print func ^ " evaluated to " ^ actual ^ ", expected " ^ expected);
  (* Inputs are immutable registers throughout compiled execution. *)
  List.iteri
    (fun i input ->
      assert (Utils.Number.equal input (List.assoc ("x" ^ string_of_int i) env)))
    input

let api_check text input =
  let func = ok (Code.parse text) and args = ok (Ainput.parse input) in
  let expected = run_source (ok (make func args)) in
  let code = ok (Goto_lang.Code.parse (ok (C.api.compile_code text))) in
  let env = ok (Goto_lang.Ainput.parse (ok (C.api.encode_ainput input))) in
  let result = run_code code env in
  assert (ok (C.api.decode_foutput (Goto_lang.Ainput.print result)) = expected)

let () =
  check Zero [];
  for n = 0 to 5 do
    check Succ (numbers [ n ]);
    check (Comp (Succ, [ Succ ])) (numbers [ n ]);
    check predecessor (numbers [ n ]);
    check (Mu (Proj (2, 0))) (numbers [ n ]);
    check (Mu subtract) (numbers [ n ]);
    for m = 0 to 5 do
      check (Proj (2, 1)) (numbers [ n; m ]);
      check add (numbers [ n; m ]);
      check subtract (numbers [ n; m ]);
      check multiply (numbers [ n; m ]);
      check
        (Comp (Proj (2, 1), [ Proj (2, 0); Comp (Succ, [ Proj (2, 1) ]) ]))
        (numbers [ n; m ])
    done
  done;
  check (Comp (Succ, [ Comp (Succ, [ Zero ]) ])) [];
  check (Mu (Mu (Proj (2, 0)))) [];
  check (Comp (Succ, [ Mu subtract ])) (numbers [ 4 ]);
  check
    (Prim (Mu (Proj (2, 0)), Comp (Succ, [ Proj (3, 0) ])))
    (numbers [ 3; 7 ]);
  check Succ
    [
      ok
        (Utils.Number.of_string
           "99999999999999999999999999999999999999999999999999");
    ];
  check
    (Proj (2, 1))
    (List.map
       (fun text -> ok (Utils.Number.of_string text))
       [ "123456789012345678901234567890"; "987654321098765432109876543210" ]);
  api_check "PRIM[z:PROJ[1,0] s:COMP[SUCC: PROJ[3,0]]]" "(2,4)";
  api_check
    "let p = PRIM[z:ZERO s:PROJ[2,1]]. let sub = PRIM[z:PROJ[1,0] s:COMP[p: \
     PROJ[3,0]]]. MUOP[sub]"
    "(4)";
  api_check "COMP[PROJ[2,1]: (SUCC,COMP[SUCC: SUCC])]" "(5)";
  assert (
    ok (C.api.compile_code "SUCC")
    = "clr __one;\ninc __one;\ncpy y <- x0;\ninc y;");
  assert (ok (C.api.encode_ainput "(2,3)") = "x0 = 2\nx1 = 3");
  assert (ok (C.api.encode_ainput "()") = "");
  assert (ok (C.api.decode_foutput "other = 3") = "0");
  assert (ok (C.api.encode_rinput " ") = "");
  assert (ok (C.api.decode_routput "") = "");
  error (C.api.encode_rinput "0");
  error (C.api.decode_routput "0");
  error (C.api.compile_code "PROJ[1,2]");
  error (C.compile (Prim (Zero, Succ)));
  error (C.compile (Comp (Succ, [])));
  error (C.api.encode_ainput "(-1)");
  error (C.api.decode_foutput "y = invalid");
  (* A nonterminating mu remains a running target machine. *)
  let state = ok (Goto_lang.make (ok (C.compile (Mu Succ))) []) in
  let rec diverges steps state =
    if steps > 0 then
      match ok (Goto_lang.step state ()) with
      | Halt _ -> failwith "A divergent mu search halted"
      | Continue { next; _ } ->
          let restored = ok (Goto_lang.restore (Goto_lang.snapshot next)) in
          assert (restored = next);
          diverges (steps - 1) restored
  in
  diverges 100 state;
  print_endline "recursive_function -> goto: all compiler tests passed"
