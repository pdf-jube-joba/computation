module M = Pure_lisp.Main
module S = Pure_lisp.Sexp

let get = function Ok value -> value | Error message -> failwith message
let check message condition = if not condition then failwith message
let start source = get (M.make (get (M.Code.parse source)) ())

let rec run remaining state =
  if remaining = 0 then Error "Step limit exceeded"
  else
    (* Exercise saved continuations, including a real JSON encoding boundary. *)
    let saved = M.Snapshot.to_yojson state in
    let restored =
      get
        (M.Snapshot.of_yojson
           (Yojson.Safe.from_string (Yojson.Safe.to_string saved)))
    in
    check "snapshot roundtrip" (state = restored);
    check "render roundtrip" (M.render state = M.render restored);
    match M.step restored () with
    | Error message -> Error message
    | Ok (Utils_runtime.Machine.Halt value) -> Ok (M.Foutput.print value)
    | Ok (Utils_runtime.Machine.Continue { next; _ }) ->
        run (remaining - 1) next

let expect source expected =
  let actual = get (run 10000 (start source)) in
  check
    (source ^ ": expected " ^ expected ^ ", got " ^ actual)
    (actual = expected)

let reject source fragment =
  match run 10000 (start source) with
  | Ok value -> failwith (source ^ ": unexpectedly returned " ^ value)
  | Error message ->
      check
        (source ^ ": wrong error: " ^ message)
        (String.starts_with ~prefix:fragment message)

let () =
  List.iter
    (fun (source, expected) -> expect source expected)
    [
      ("nil", "nil");
      ("()", "nil");
      ("t", "t");
      ("'unbound", "unbound");
      ("''x", "(quote x)");
      ("'(a b . c)", "(a b . c)");
      ("(atom nil)", "t");
      ("(atom 'a)", "t");
      ("(atom '(a))", "nil");
      ("(eq 'a 'a)", "t");
      ("(eq 'a 'b)", "nil");
      ("(eq nil '())", "t");
      ("(eq '(a) '(a))", "nil");
      ("(cons 'a 'b)", "(a . b)");
      ("(cons 'a '(b c))", "(a b c)");
      ("(car '(a . b))", "a");
      ("(cdr '(a . b))", "b");
      ("(car nil)", "nil");
      ("(cdr nil)", "nil");
      ("(cond)", "nil");
      ("(cond (nil missing))", "nil");
      ("(cond (nil missing) ('yes 'chosen) (t missing))", "chosen");
      ("((lambda () 'ok))", "ok");
      ("((lambda (x y) (cons x y)) 'a 'b)", "(a . b)");
      ("((lambda (f) (f '(a b))) car)", "a");
      ("((cond (t car)) '(a b))", "a");
      ("((lambda (car) (car 'a 'b)) cons)", "(a . b)");
      ( "((lambda (x) ((lambda (f) ((lambda (x) (f)) 'inner)) (lambda () x))) \
         'outer)",
        "inner" );
      ( "((lambda (f) ((lambda (x) (f)) 'caller)) ((lambda (x) (lambda () x)) \
         'definition))",
        "caller" );
      ( "((lambda (x) (cons ((lambda (x) x) 'inner) (cons x nil))) 'outer)",
        "(inner outer)" );
      ( "((lambda (x) (cons (((lambda (x) (lambda (y) y)) 'inner) x) x)) 'outer)",
        "(outer . outer)" );
      ("((lambda (x) (cond (((lambda (x) t) 'inner) x))) 'outer)", "outer");
      ("(eval ''a)", "a");
      ("(eval '(cons 'a '(b)))", "(a b)");
      ("(eval (cons 'cons (cons ''a (cons ''b nil))))", "(a . b)");
      ("((lambda (x) (eval 'x)) 'dynamic)", "dynamic");
      ("((lambda (x) (eval ((lambda (x) 'x) 'inner))) 'outer)", "outer");
      ("((lambda (e x) (e 'x)) eval 'aliased)", "aliased");
      ("('(lambda (x) x) 'data)", "data");
      ("(car (lambda (x) x))", "lambda");
      ("((eval '(lambda (x) x)) 'ok)", "ok");
      ( "((label append (lambda (xs ys) (cond ((eq xs nil) ys) (t (cons (car \
         xs) (append (cdr xs) ys)))))) '(a b) '(c d))",
        "(a b c d)" );
      ( "((lambda (x) ((label walk (lambda (xs) (cond ((eq xs nil) x) (t (walk \
         (cdr xs)))))) '(a b))) 'dynamic)",
        "dynamic" );
    ];
  List.iter
    (fun (source, fragment) -> reject source fragment)
    [
      ("missing", "Unbound symbol");
      ("(eval 'missing)", "Unbound symbol");
      ("(((lambda (x) (lambda () x)) 'gone))", "Unbound symbol");
      ("((lambda (x y) y) 'a x)", "Unbound symbol");
      ("(cons missing also-missing)", "Unbound symbol: missing");
      ("(missing also-missing)", "Unbound symbol: missing");
      ("(car 'a)", "car/cdr expects");
      ("(cdr 'a)", "car/cdr expects");
      ("(quote)", "quote expects");
      ("(quote a b)", "quote expects");
      ("(eval)", "Wrong number");
      ("(eval 'a 'b)", "Wrong number");
      ("(cons 'a)", "Wrong number");
      ("((lambda (x) x))", "Wrong number");
      ("((lambda () t) 'a)", "Wrong number");
      ("(lambda (x x) x)", "Duplicate parameter");
      ("(lambda (nil) t)", "Expected a variable");
      ("(lambda (t) t)", "Expected a variable");
      ("(lambda (x . y) x)", "Expected a proper list");
      ("(lambda (x))", "Expected (lambda");
      ("(label f 'x)", "Expected (lambda");
      ("(cond (t))", "Each cond clause");
      ("(cons 'a . b)", "Expected a proper list");
      ("('a 'b)", "Not a function");
    ];
  List.iter
    (fun source ->
      let value = get (S.parse source) in
      check
        ("S-expression roundtrip: " ^ source)
        (get (S.parse (S.print value)) = value))
    [
      "()";
      "nil";
      "'a";
      "''a";
      "(a b . c)";
      "(a . (b . nil))";
      "(foo-bar + 123 ... λ)";
      "; comment\n'(a ; comment\n b) ; end";
    ];
  List.iter
    (fun source ->
      check
        ("Invalid syntax accepted: " ^ source)
        (Result.is_error (S.parse source)))
    [
      "";
      "'";
      "(";
      ")";
      "a b";
      ".";
      "(. a)";
      "(a .)";
      "(a . b c)";
      "(a . b";
      "(a))";
    ];
  check "invalid snapshot accepted"
    (Result.is_error (M.Snapshot.of_yojson `Null));
  let loop = start "((label loop (lambda () (loop))))" in
  let rec bounded remaining state =
    if remaining > 0 then
      match get (M.step state ()) with
      | Utils_runtime.Machine.Continue { next; _ } ->
          bounded (remaining - 1) next
      | Utils_runtime.Machine.Halt _ ->
          failwith "Recursive loop unexpectedly halted"
  in
  bounded 1000 loop;
  print_endline
    "Passed Pure Lisp evaluation, dynamic scope, eval, parser and snapshot \
     checks."
