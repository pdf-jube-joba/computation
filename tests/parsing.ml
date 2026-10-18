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

let () =
  nested_groups ();
  optional_groups ();
  failed_groups ();
  print_endline "Passed parser group checks."
