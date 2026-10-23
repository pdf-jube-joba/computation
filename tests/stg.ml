module M = Stg.Main
module C = Stg.Syntax
module Step = Utils_runtime.Machine

let get = function Ok v -> v | Error e -> failwith e
let check message b = if not b then failwith message
let start source = get (M.make (get (M.Code.parse source)) ())

let rec run fuel s =
  if fuel = 0 then Error "Step limit exceeded"
  else
    let json = M.Snapshot.to_yojson s in
    let restored =
      get
        (M.Snapshot.of_yojson
           (Yojson.Safe.from_string (Yojson.Safe.to_string json)))
    in
    check "snapshot roundtrip" (s = restored);
    check "render roundtrip" (M.render s = M.render restored);
    let result = M.step restored () in
    check "step mutated saved state" (M.Snapshot.to_yojson s = json);
    match result with
    | Error e -> Error e
    | Ok (Step.Halt v) -> Ok (v, s)
    | Ok (Step.Continue { next; _ }) -> run (fuel - 1) next

let expect source expected =
  let code = get (M.Code.parse source) in
  check "code roundtrip" (get (M.Code.parse (M.Code.print code)) = code);
  let v, s = get (run 20000 (start source)) in
  let actual = M.Foutput.print v in
  check
    ("Expected " ^ expected ^ ", got " ^ actual ^ " in " ^ source)
    (actual = expected);
  check "output roundtrip" (get (M.Foutput.parse actual) = v);
  s

let reject source fragment =
  match run 1000 (start source) with
  | Ok _ -> failwith ("Unexpected success: " ^ source)
  | Error e ->
      check ("Wrong error: " ^ e) (String.starts_with ~prefix:fragment e)

let () =
  List.iter
    (fun (source, expected) -> ignore (expect source expected))
    [
      ({|main = \fun [] \u [] -> 42|}, "42");
      ({|main = \fun [] \n [] -> \sub(1, 3)|}, "0");
      ( {|main = \fun [] \u [] -> \add(999999999999999999999999999999, 1)|},
        "1000000000000000000000000000000" );
      ({|id = \fun [] \n [x] -> x(); main = \fun [] \u [] -> id(7)|}, "7");
      (* Unused arguments and constructor fields must not be forced. *)
      ( {|loop = \fun [] \u [] -> loop();
       first = \fun [] \n [x, y] -> x();
       main = \fun [] \u [] -> first(9, loop)|},
        "9" );
      ( {|loop = \fun [] \u [] -> loop();
       main = \fun [] \u [] -> \case \con Cons(8, loop) \of { Cons(x, xs) -> x() }|},
        "8" );
      (* Constructor return and heap references remain in WHNF. *)
      ( {|loop = \fun [] \u [] -> loop(); main = \fun [] \u [] -> \con Cons(1, loop)|},
        "\\con Cons(1, @0)" );
      ( {|f = \fun [] \n [x] -> x(); main = \fun [] \u [] -> f()|},
        "\\function @0" );
      (* PAP allocation, update, and later entry. *)
      ( {|add = \fun [] \n [x, y] -> \add(x, y);
       inc = \fun [] \u [] -> add(1);
       main = \fun [] \u [] -> inc(41)|},
        "42" );
      (* Overapplication passes the leftover arguments to a returned closure. *)
      ( {|id = \fun [] \n [x] -> x();
       choose = \fun [] \n [x] -> id();
       main = \fun [] \u [] -> choose(0, 42)|},
        "42" );
      (* A case must save arguments belonging to its enclosing application. *)
      ( {|id = \fun [] \n [x] -> x();
       choose = \fun [] \n [x] -> \case x() \of { 0 -> id(); \default -> id() };
       main = \fun [] \u [] -> choose(0, 17)|},
        "17" );
      (* Both a function-valued thunk and a PAP-valued thunk are updated. *)
      ( {|add = \fun [] \n [x, y] -> \add(x, y);
       f = \fun [] \u [] -> g(); g = \fun [] \u [] -> add(2);
       main = \fun [] \u [] -> \case f(3) \of { \default x ->
         \case g(4) \of { \default y -> \add(x, y) } }|},
        "11" );
      (* Explicit captures preserve the allocation environment. *)
      ( {|make = \fun [] \n [x] -> \let { f = \fun [x] \n [y] -> \add(x, y) } \in f();
       main = \fun [] \u [] -> \case make(10) \of { \default f -> f(7) }|},
        "17" );
      (* A nonrecursive RHS sees the outer binding with the same name. *)
      ( {|id = \fun [] \n [x] -> \let { x = \fun [x] \u [] -> x() } \in x();
       main = \fun [] \u [] -> id(12)|},
        "12" );
      (* Local bindings may shadow globals if captured explicitly. *)
      ( {|x = \fun [] \u [] -> 99;
       f = \fun [] \n [x] -> \let { g = \fun [x] \u [] -> x() } \in g();
       main = \fun [] \u [] -> f(12)|},
        "12" );
      (* Nonmatching default can precede a specific alternative. *)
      ({|main = \fun [] \u [] -> \case 5 \of { \default -> 0; 5 -> 1 }|}, "1");
      ( {|main = \fun [] \u [] -> \case \con Pair(3, 4) \of {
         \default p -> \case p() \of { Pair(x, y) -> \mul(x, y) } }|},
        "12" );
      (* A cyclic constructor is productive, unlike a recursive thunk entry. *)
      ( {|main = \fun [] \u [] -> \letrec {
         ones = \fun [ones] \n [] -> \con Cons(1, ones)
       } \in \case ones() \of { Cons(x, xs) ->
         \case xs() \of { Cons(y, ys) -> \add(x, y) } }|},
        "2" );
      (* Global recursion, unboxing and nested continuations. *)
      ( {|fact = \fun [] \n [n] -> \case n() \of {
         0 -> 1; \default k -> \case \sub(k, 1) \of {
           \default m -> \case fact(m) \of { \default r -> \mul(k, r) } } };
       main = \fun [] \u [] -> fact(6)|},
        "720" );
      (* Mutually recursive local closures capture each other's addresses. *)
      ( {|main = \fun [] \u [] -> \letrec {
       even = \fun [odd] \n [n] -> \case n() \of { 0 -> 1; \default k ->
         \case \sub(k, 1) \of { \default m -> odd(m) } };
       odd = \fun [even] \n [n] -> \case n() \of { 0 -> 0; \default k ->
         \case \sub(k, 1) \of { \default m -> even(m) } }
       } \in even(8)|},
        "1" );
      ( {|main = \fun [] \u [] -> \case \eq(2, 2) \of {
       1 -> \lt(2, 3); \default -> 0 }|},
        "1" );
    ];
  let shared =
    expect
      {|main = \fun [] \u [] -> \let {
    x = \fun [] \u [] -> \let { box = \fun [] \n [] -> \con Box(21) } \in box()
    } \in \case x() \of { Box(a) -> \case x() \of { Box(b) -> \add(a, b) } }|}
      "42"
  in
  check "shared thunk was evaluated twice" (shared.next_addr = 3);
  check "shared thunk was not updated"
    (match List.assoc 1 shared.heap with
    | M.Evaluated (M.Data _) -> true
    | _ -> false);
  let unshared =
    expect
      {|main = \fun [] \u [] -> \let {
    x = \fun [] \n [] -> \let { box = \fun [] \n [] -> \con Box(21) } \in box()
    } \in \case x() \of { Box(a) -> \case x() \of { Box(b) -> \add(a, b) } }|}
      "42"
  in
  check "non-updatable closure was shared" (unshared.next_addr = 4);
  List.iter
    (fun (source, error) -> reject source error)
    [
      ({|main = \fun [] \u [] -> main()|}, "Black hole");
      ( {|main = \fun [] \u [] -> \letrec { x = \fun [y] \u [] -> y(); y = \fun [x] \u [] -> x() } \in x()|},
        "Black hole" );
      ({|main = \fun [] \u [] -> \case 1 \of { 2 -> 0 }|}, "No matching");
      ( {|main = \fun [] \u [] -> \case \con Pair(1, 2) \of { Pair(x) -> x() }|},
        "Constructor pattern arity" );
      ( {|x = \fun [] \u [] -> 1; main = \fun [] \u [] -> \add(x, 2)|},
        "Primitive operands" );
      ( {|f = \fun [] \n [x] -> x(); main = \fun [] \u [] -> f(1, 2)|},
        "Cannot apply" );
      ( {|main = \fun [] \u [] -> \let { x = \fun [] \n [] -> \con Nil() } \in x(1)|},
        "Cannot apply" );
    ];
  List.iter
    (fun source ->
      check
        ("Invalid code accepted: " ^ source)
        (Result.is_error (M.Code.parse source)))
    [
      "";
      {|main = fun [] n [] -> 0|};
      {|other = \fun [] \n [] -> 0|};
      {|main = \fun [] \u [x] -> x()|};
      {|main = \fun [] \n [x] -> x()|};
      {|main = \fun [] \u [] -> missing()|};
      {|main = \fun [] \u [] -> 0; main = \fun [] \u [] -> 1|};
      {|f = \fun [] \n [x, x] -> x(); main = \fun [] \u [] -> 0|};
      {|main = \fun [] \u [] -> \let { x = \fun [x] \u [] -> x() } \in x()|};
      {|f = \fun [] \n [x] -> \let { g = \fun [] \u [] -> x() } \in g(); main = \fun [] \u [] -> 0|};
      {|x = \fun [] \u [] -> 99; f = \fun [] \n [x] -> \let { g = \fun [] \u [] -> x() } \in g(); main = \fun [] \u [] -> f(12)|};
      {|main = \fun [] \u [] -> \case 0 \of {}|};
      {|main = \fun [] \u [] -> \case 0 \of { 0 -> 1; 0 -> 2 }|};
      {|main = \fun [] \u [] -> \case 0 \of { \default x -> 1; \default -> 2 }|};
      {|main = \fun [] \u [] -> \case \con Pair(1, 2) \of { Pair(x, x) -> x() }|};
      {|main = \fun [] \u [] -> \add(1)|};
      {|main = \fun [] \u [] -> \add(f(), 2)|};
    ];
  let initial = start {|main = \fun [] \u [] -> 42|} in
  let invalid s =
    check "invalid snapshot accepted"
      (Result.is_error (M.Snapshot.of_yojson (M.Snapshot.to_yojson s)))
  in
  invalid { initial with next_addr = -1 };
  invalid { initial with next_addr = 10 };
  invalid { initial with heap = [] };
  invalid { initial with globals = [] };
  invalid { initial with control = M.Enter (M.Addr 100) };
  invalid { initial with heap = [ (0, M.Blackhole) ] };
  invalid { initial with stack = [ M.Update (0, []) ] };
  invalid { initial with heap = [ (0, M.Pap (0, [ M.Int C.N.one ])) ] };
  invalid { initial with control = M.Return (M.Function 0) };
  check "malformed snapshot accepted"
    (Result.is_error (M.Snapshot.of_yojson `Null));
  (* Non-updatable recursion remains a sequence of machine steps. *)
  let rec bounded n s =
    if n > 0 then
      match get (M.step s ()) with
      | Step.Continue { next; _ } -> bounded (n - 1) next
      | Step.Halt _ -> failwith "Recursive loop halted"
  in
  bounded 1000 (start {|main = \fun [] \n [] -> main()|});
  print_endline
    "Passed STG evaluation, sharing, applications, recursion, parser and \
     snapshot checks."
