open Utils.Machine
open Recursive_function
module L = Lambda_calculus
module C = Recursive_function_lambda_calculus
module Typed_contract : Utils.Compiler.S = C.Compiler

let ok = function Ok value -> value | Error message -> failwith message
let error = function Error _ -> () | Ok _ -> failwith "Expected an error"
let numbers = List.map Utils.Number.of_int
let numeral n = ok (C.number_to_lambda_term (Utils.Number.of_int n))
let add = Prim (Proj (1, 0), Comp (Succ, [ Proj (3, 0) ]))
let predecessor = Prim (Zero, Proj (2, 1))
let subtract = Prim (Proj (1, 0), Comp (predecessor, [ Proj (3, 0) ]))
let zero_unary = Prim (Zero, Proj (2, 0))
let multiply = Prim (zero_unary, Comp (add, [ Proj (3, 0); Proj (3, 2) ]))

let normalize term =
  let rec run fuel term =
    if L.redex_count term = 0 then term
    else if fuel = 0 then failwith "Lambda evaluation step limit exceeded"
    else
      match ok (L.step term 0) with
      | Halt output -> output
      | Continue { next; _ } -> run (fuel - 1) next
  in
  run 100000 term

let alpha_equal left right =
  let rec equal env left right =
    match (left, right) with
    | L.Var a, L.Var b -> (
        match List.assoc_opt a.id env with
        | Some id -> id = b.id
        | None -> a.id = b.id)
    | L.Abs (a, x), L.Abs (b, y) -> equal ((a.id, b.id) :: env) x y
    | L.App (a, b), L.App (x, y) -> equal env a x && equal env b y
    | _ -> false
  in
  equal [] left right

let check func input =
  let expected = ok (evaluate (ok (make func input)).process) in
  let term = ok (C.compile func) in
  let arguments = ok (C.encode_ainput input) in
  let normal = normalize (ok (L.make term arguments)) in
  let actual = ok (C.decode_foutput normal) in
  if not (Utils.Number.equal expected actual) then
    failwith (print func ^ " evaluated to " ^ actual ^ ", expected " ^ expected);
  (* Exercise the text boundary, whose parser allocates IDs independently. *)
  let reparsed = ok (L.parse_code (L.print term)) in
  let normal = normalize (ok (L.make reparsed arguments)) in
  assert (ok (C.decode_foutput normal) = expected);
  assert (ok (L.restore (L.snapshot normal)) = normal)

let api_check text input =
  let func = ok (parse_code text) and args = ok (Ainput.parse input) in
  let expected = ok (evaluate (ok (make func args)).process) in
  let term = ok (L.parse_code (ok (C.api.compile_code text))) in
  let args = ok (L.Ainput.parse (ok (C.api.encode_ainput input))) in
  let index = ok (L.Rinput.parse (ok (C.api.encode_rinput ""))) in
  assert (index = 0);
  let normal = normalize (ok (L.make term args)) in
  assert (ok (C.api.decode_foutput (L.print normal)) = expected)

let () =
  for n = 0 to 5 do
    assert (C.lambda_term_to_number (numeral n) = Some (Utils.Number.of_int n));
    let argument = [ numeral n ] in
    assert (
      ok (C.decode_foutput (normalize (ok (L.make (C.succ ()) argument))))
      = Utils.Number.of_int (n + 1));
    assert (
      ok (C.decode_foutput (normalize (ok (L.make (C.pred ()) argument))))
      = Utils.Number.of_int (max 0 (n - 1)));
    let boolean = normalize (ok (L.make (C.is_zero ()) argument)) in
    assert (
      alpha_equal boolean
        (if n = 0 then C.true_lambda () else C.false_lambda ()))
  done;
  check Zero [];
  for n = 0 to 3 do
    check Succ (numbers [ n ]);
    check (Comp (Succ, [ Succ ])) (numbers [ n ]);
    check predecessor (numbers [ n ]);
    check (Mu (Proj (2, 0))) (numbers [ n ]);
    check (Mu subtract) (numbers [ n ]);
    for m = 0 to 3 do
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
  check (Comp (Succ, [ Mu subtract ])) (numbers [ 3 ]);
  check
    (Prim (Mu (Proj (2, 0)), Comp (Succ, [ Proj (3, 0) ])))
    (numbers [ 3; 2 ]);
  api_check "PRIM[z:PROJ[1,0] s:COMP[SUCC: PROJ[3,0]]]" "(2,4)";
  api_check
    "let p = PRIM[z:ZERO s:PROJ[2,1]]. let sub = PRIM[z:PROJ[1,0] s:COMP[p: \
     PROJ[3,0]]]. MUOP[sub]"
    "(3)";
  api_check "COMP[PROJ[2,1]: (SUCC,COMP[SUCC: SUCC])]" "(5)";
  (* Helper inputs may come from different parses with colliding IDs. *)
  let composed =
    C.composition 1 [ C.succ (); C.succ () ] (Option.get (C.projection 2 1))
  in
  assert (
    ok (C.decode_foutput (normalize (ok (L.make composed [ numeral 2 ])))) = "3");
  let primitive =
    C.primitive_recursion 1 (numeral 0) (Option.get (C.projection 2 1))
  in
  assert (
    ok (C.decode_foutput (normalize (ok (L.make primitive [ numeral 3 ]))))
    = "2");
  let mu = C.mu_recursion 1 (ok (C.compile subtract)) in
  assert (ok (C.decode_foutput (normalize (ok (L.make mu [ numeral 2 ])))) = "2");
  assert (Option.is_none (C.projection 1 1));
  assert (ok (C.api.encode_ainput "()") = "");
  assert (ok (C.api.encode_rinput " ") = "0");
  assert (ok (C.api.decode_routput "") = "");
  assert (ok (C.api.decode_foutput "\\s.\\s.s") = "0");
  assert (ok (C.api.decode_foutput "\\foo.\\bar.(foo (foo bar))") = "2");
  error (C.api.encode_rinput "0");
  error (C.api.decode_routput "0");
  error (C.api.decode_foutput "\\s.\\s.(s s)");
  error (C.api.decode_foutput "\\s.\\z.(free z)");
  error (C.api.decode_foutput "\\x.x");
  error (C.api.decode_foutput "(\\x.x) (\\s z.z)");
  error (C.compile (Prim (Zero, Succ)));
  error (C.compile (Mu Zero));
  error (C.api.compile_code "PROJ[1,2]");
  error (C.api.encode_ainput "(-1)");
  error (C.api.encode_ainput "(99999999999999999999999999999999999999999999)");
  let same : L.var = { id = 0; name = "s" } in
  error (C.decode_foutput (L.Abs (same, L.Abs (same, L.Var same))));
  (* The first argument of Y is allowed to be an open term. *)
  let f : L.var = { id = 0; name = "f_test" } in
  let term = ok (L.make (C.y_combinator ()) [ L.Var f ]) in
  let term = ok (L.reduce term 0) in
  assert (L.redex_count term > 0);
  (* Nonterminating search keeps reducing; it is not a numeral or a stub. *)
  let rec diverges steps term =
    if steps > 0 then
      match ok (L.step term 0) with
      | Halt _ -> failwith "A divergent mu search halted"
      | Continue { next; _ } -> diverges (steps - 1) next
  in
  diverges 100 (ok (C.compile (Mu Succ)));
  print_endline "recursive_function -> lambda: all compiler tests passed"
