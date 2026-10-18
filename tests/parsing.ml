module P = Utils_parsing.Parser

let get = Utils_base.Error.get
let check message condition = if not condition then failwith message

let nested_groups () =
  let parsed =
    P.parse
      (fun p ->
        P.group p P.Parentheses (fun p ->
            let numbers =
              P.group p P.Brackets (fun p ->
                  P.separated_list p ~separator:',' P.integer)
            in
            P.expect_symbol p ',';
            let last = P.group p P.Braces P.integer in
            (numbers, last)))
      "([1, 2], {3})"
  in
  check "nested groups keep separate scopes" (get parsed = ([ 1; 2 ], 3));
  let text =
    P.parse
      (fun p -> P.group p P.Parentheses P.string_literal)
      "(/* [} */ \"[)]\")"
  in
  check "brackets in comments and strings are ignored" (get text = "[)]");
  List.iter
    (fun text ->
      check "unmatched brackets rejected"
        (Result.is_error (P.protect (fun () -> P.create text))))
    [ "([)]"; "(1"; "1)"; "{[1]}]" ]

let optional_groups () =
  let p = P.create "[1] (2)" in
  let before = P.offset p in
  check "lookahead distinguishes delimiters" (P.is_group p P.Brackets);
  check "absent optional group returns None"
    (P.bump_group_if p P.Parentheses P.integer = None);
  check "absent optional group preserves position" (P.offset p = before);
  check "required group consumes its contents"
    (P.group p P.Brackets P.integer = 1);
  check "present optional group consumes its contents"
    (P.bump_group_if p P.Parentheses P.integer = Some 2);
  check "groups consume their closing brackets" (P.eof p);
  check "no group at end of scope" (not (P.is_group p P.Parentheses))

let failed_groups () =
  let p = P.create "(1 2) 3" in
  let before = P.offset p in
  check "group rejects unconsumed contents"
    (Result.is_error (P.protect (fun () -> P.group p P.Parentheses P.integer)));
  check "failed group preserves parent position" (P.offset p = before);
  check "failed group can be read again"
    (P.group p P.Parentheses (fun p -> P.many p P.integer) = [ 1; 2 ]);
  check "group cannot consume outside its scope" (P.integer p = 3);
  let p = P.create "(word)" in
  check "errors inside optional groups propagate"
    (Result.is_error
       (P.protect (fun () -> P.bump_group_if p P.Parentheses P.integer)));
  check "optional group error preserves parent position"
    (P.is_group p P.Parentheses)

let keywords () =
  let p = P.create {|if \if ifz \ifz|} in
  check "ordinary keyword spelling remains an identifier"
    (Utils_names.Identifier.to_string (P.identifier p) = "if");
  check "prefixed word is a distinct token" (P.peek p = P.Keyword "if");
  check "identifier rejects a keyword"
    (Result.is_error (P.protect (fun () -> P.identifier p)));
  check "failed keyword sequence preserves its cursor"
    (not (P.bump_if p {|\if otherwise|}));
  P.expect_keyword p "if";
  check "keyword matching uses a complete word" (not (P.bump_if_keyword p "if"));
  ignore (P.identifier p);
  check "keyword names exclude their prefix" (P.keyword p = "ifz");
  P.expect_eof p;
  check "unprefixed word is not accepted as a keyword"
    (Result.is_error (P.parse (fun p -> P.expect_keyword p "if") "if"));
  check "prefix must be adjacent to its word"
    (Result.is_error (P.parse (fun p -> P.expect_keyword p "if") {|\ if|}));
  check "keywords in strings and comments remain literal data"
    (get (P.parse P.string_literal {|/* \if */ "\\fun" // \else|}) = {|\fun|})

let () =
  nested_groups ();
  optional_groups ();
  failed_groups ();
  keywords ();
  print_endline "Passed parser group checks."
