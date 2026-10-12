open Utils
module P = Parser

let binary op a b = "(" ^ a ^ op ^ b ^ ")"
let unary op a = "(" ^ op ^ a ^ ")"

let expression =
  P.expression
    ~prefix:[ ("~", 4, unary "~"); ("not", 4, unary "not ") ]
    ~infix:
      [
        ("==", 0, 1, binary "==");
        ("=", 0, 1, binary "=");
        ("+", 1, 2, binary "+");
        ("-", 1, 2, binary "-");
        ("*", 3, 4, binary "*");
        ("^", 5, 5, binary "^");
      ]
    ~atom:(fun read p ->
      if P.peek p = P.Symbol '(' then P.group p ~opening:'(' read
      else P.identifier p)

let () =
  List.iter
    (fun (source, expected) -> assert (P.parse expression source = Ok expected))
    [
      ("a-b-c", "((a-b)-c)");
      ("a^b^c", "(a^(b^c))");
      ("a+b*c", "(a+(b*c))");
      ("(a+b)*c", "((a+b)*c)");
      ("~a^b*c", "((~(a^b))*c)");
      ("~~a+b", "((~(~a))+b)");
      ("not a+b", "((not a)+b)");
      ("a==b", "(a==b)");
      ("a=b", "(a=b)");
    ];
  (* A low-precedence operator or statement keyword must remain available to
     the caller after a higher-precedence recursive read. *)
  assert (
    P.parse
      (fun p ->
        let value = expression p in
        P.expect_keyword p "goto";
        let offset = P.integer p in
        (value, offset))
      "a*b+c goto 2"
    = Ok ("((a*b)+c)", 2));
  List.iter
    (fun source -> assert (Result.is_error (P.parse expression source)))
    [ "a+"; "~"; "a=="; "a b"; "(a+)"; "()" ];
  let p = P.create "name := value" in
  assert (
    Result.is_error
      (P.attempt p (fun p ->
           ignore (P.identifier p);
           P.expect p ":+")));
  assert (P.identifier p = "name");
  assert (P.attempt p (fun p -> P.expect p ":=") = Ok ());
  assert (P.identifier p = "value");
  P.expect_eof p;
  let chain ?(allow_trailing = false) text =
    P.parse
      (fun p ->
        P.group p ~opening:'{' (fun p ->
            P.chain_left ~allow_trailing p ~separator:';' ~combine:(binary ";")
              P.identifier))
      text
  in
  assert (chain "{a;b;c}" = Ok "((a;b);c)");
  assert (chain ~allow_trailing:true "{a;b;}" = Ok "(a;b)");
  assert (Result.is_error (chain "{a;}"));
  assert (Result.is_error (chain "{}"));
  assert (Result.is_error (chain ~allow_trailing:true "{a;;}"));
  let relation =
    P.binary
      ~operators:[ ("==", "=="); ("<", "<") ]
      ~operand:P.identifier
      ~make:(fun a op b -> binary op a b)
  in
  assert (P.parse relation "a==b" = Ok "(a==b)");
  assert (P.parse relation "a<b" = Ok "(a<b)");
  List.iter
    (fun source -> assert (Result.is_error (P.parse relation source)))
    [ "a"; "a=="; "a==b==c"; "a=b" ];
  assert (
    Result.is_error
      (P.parse
         (P.expression
            ~infix:[ ("", 0, 1, binary "") ]
            ~atom:(fun _ p -> P.identifier p))
         "a"));
  print_endline "Shared expression parser tests passed"
