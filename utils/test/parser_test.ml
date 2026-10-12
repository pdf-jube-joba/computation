open Utils
module P = Parser

let bad result = assert (Result.is_error result)

let () =
  (* A source with all delimiter kinds, brackets in literals/comments, and a
     natural too large for an OCaml int. Only syntax brackets form groups. *)
  let source =
    "( [000123456789012345678901234567890] {\"[})\" '\\n'} /* ] */ ) tail"
  in
  let read p =
    let values =
      P.group p ~opening:'(' (fun p ->
          let n = P.group p ~opening:'[' P.natural in
          let s, c =
            P.group p ~opening:'{' (fun p ->
                let s = P.string_literal p in
                let c = P.character_literal p in
                (s, c))
          in
          assert (P.eof p);
          assert (P.bump p = P.Eof);
          assert (P.bump p = P.Eof);
          (n, s, c))
    in
    assert (P.identifier p = "tail");
    values
  in
  assert (
    P.parse ~character_literals:true read source
    = Ok ("123456789012345678901234567890", "[})", "\n"));
  let p = P.create "([name] {})" in
  (match P.tree p with
  | [
   P.Group
     {
       delimiter = P.Parentheses;
       opening_offset = 0;
       closing_offset = 10;
       children =
         [
           P.Group
             {
               delimiter = P.Brackets;
               opening_offset = 1;
               closing_offset = 6;
               children = [ P.Atom { token = P.Ident "name"; offset = 2 } ];
             };
           P.Group
             {
               delimiter = P.Braces;
               opening_offset = 8;
               closing_offset = 9;
               children = [];
             };
         ];
     };
  ] ->
      ()
  | _ -> failwith "Unexpected token tree");
  (* Even a callback that consumes nothing must reject malformed input during
     initialization. The lexer alone still accepts arbitrary symbol sequences. *)
  List.iter
    (fun source -> bad (P.parse (fun _ -> ()) source))
    [ "("; "["; "{"; ")"; "]"; "}"; "([)]"; "{]"; "[{}" ];
  assert (fst (Lexer.tokenize "([)]").(0) = Lexer.Symbol '(');
  assert (
    P.parse (fun _ -> ()) "([)]"
    = Error "Expected ']' for '[' at byte 1, found ')' at byte 2");
  assert (
    P.parse (fun _ -> ()) "\"あ\" )" = Error "Unexpected closing ')' at byte 6");
  assert (
    P.parse (fun _ -> ()) "(\"あ\""
    = Error "Unclosed '(' at byte 0; expected ')' at byte 6");
  (* Group callbacks cannot consume a sibling and cannot leave tokens unread.
     In either case the parent can retry from the opening bracket. *)
  let p = P.create "(one two) outside" in
  bad (P.protect (fun () -> P.group p ~opening:'(' P.identifier));
  assert (P.peek p = P.Symbol '(');
  bad
    (P.protect (fun () ->
         P.group p ~opening:'(' (fun p ->
             ignore (P.identifier p);
             ignore (P.identifier p);
             P.identifier p)));
  assert (P.peek p = P.Symbol '(');
  assert (P.bump_group_if p ~opening:'[' P.identifier = None);
  assert (
    P.bump_group_if p ~opening:'(' (fun p -> P.many p P.identifier)
    = Some [ "one"; "two" ]);
  assert (P.identifier p = "outside");
  P.expect_eof p;
  (* A parser mixing flat token reads with scoped groups uses the same pairing. *)
  assert (
    P.parse
      (fun p ->
        P.expect_symbol p '(';
        let n = P.group p ~opening:'[' P.integer in
        P.expect_symbol p ')';
        n)
      "([0001])"
    = Ok 1);
  let p = P.create "x := (1)" in
  assert (not (P.bump_if p "x :+"));
  assert (P.identifier p = "x");
  let saved = P.checkpoint p in
  P.expect p ":=";
  P.restore p saved;
  assert (P.peek p = P.Symbol ':');
  bad (P.protect (fun () -> P.restore (P.create "other") saved));
  P.expect p ":=";
  assert (P.group p ~opening:'(' P.integer = 1);
  P.expect_eof p;
  let strict ?(allow_trailing = false) source =
    P.parse
      (fun p ->
        P.group p ~opening:'[' (fun p ->
            P.separated_list ~allow_trailing p ~separator:',' P.natural))
      source
  in
  assert (strict "[]" = Ok []);
  assert (strict "[1, 02]" = Ok [ "1"; "2" ]);
  bad (strict "[1,]");
  bad (strict "[1 2]");
  bad (strict "[,1]");
  assert (strict ~allow_trailing:true "[1,]" = Ok [ "1" ]);
  assert (
    P.parse
      (fun p ->
        P.group p ~opening:'{' (fun p -> P.many ~skip_separator:',' p P.natural))
      "{,1,, 02 3,}"
    = Ok [ "1"; "2"; "3" ]);
  bad (P.parse (fun p -> P.many p (fun _ -> ())) "name");
  bad (P.parse (fun p -> P.many_until p ~closing:';' (fun _ -> ())) "name;");
  bad (P.parse P.integer "123456789012345678901234567890");
  assert (P.parse P.integer "00000000000000000001" = Ok 1);
  let p = P.create "name 1" in
  assert (
    P.protect (fun () -> P.natural p)
    = Error "Expected natural number at byte 0");
  assert (P.identifier p = "name");
  assert (P.natural p = "1");
  assert (P.parse P.string_literal "\"a\\n\\\"b\\\\\"" = Ok "a\n\"b\\");
  bad (P.parse P.string_literal "\"unclosed");
  bad (P.parse P.string_literal "\"\\q\"");
  assert (
    P.parse ~nested_comments:true P.identifier "/* outer /* inner */ */ name"
    = Ok "name");
  bad
    (P.parse ~nested_comments:false P.identifier "/* outer /* inner */ */ name");
  print_endline "Shared parser tests passed"
