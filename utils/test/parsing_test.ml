open Utils

let bad f = assert (Result.is_error (Lexer.protect f))

let () =
  List.iter
    (fun c ->
      assert (Utf8.is_scalar c);
      assert (Utf8.scalar_length c 0 = String.length c);
      assert (Utf8.scalar ("x" ^ c ^ "y") 1 = (c, 1 + String.length c)))
    [ "a"; "é"; "あ"; "😀" ];
  assert (Utf8.valid "");
  assert (Utf8.valid "aéあ😀");
  assert (not (Utf8.is_scalar ""));
  assert (not (Utf8.is_scalar "ab"));
  List.iter
    (fun c ->
      assert (not (Utf8.valid c));
      bad (fun () -> Utf8.scalar c 0))
    [ "\128"; "\192\128"; "\237\160\128"; "\244\144\128\128"; "\226\128" ];
  List.iter
    (fun c ->
      let literal = Char_literal.to_text c in
      assert (Char_literal.read literal 0 = (c, String.length literal)))
    [ "a"; "あ"; "😀"; "\n"; "\r"; "\t"; "\\"; "'" ];
  List.iter
    (fun text -> bad (fun () -> Char_literal.read text 0))
    [ "'"; "'ab'"; "'あ"; "'\\" ];
  let p =
    Lexer.create ~character_literals:true "/* ' */ 'あ' // ' ignored\n'\\n'"
  in
  assert (Lexer.string_literal p = "あ");
  assert (Lexer.string_literal p = "\n");
  Lexer.expect_eof p;
  let tuple =
    Parse.delimited_list ~opening:'(' ~closing:')' ~separator:','
      Number.of_string_with_plus
  in
  assert (tuple "(,001,,+2,)" = Ok [ "1"; "2" ]);
  assert (tuple "()" = Ok []);
  assert (Result.is_error (tuple "(1;x)"));
  assert (Result.is_error (tuple "1,2"));
  let separated text =
    Lexer.parse
      (fun p ->
        Lexer.expect_symbol p '[';
        Lexer.separated_until p ~separator:',' ~closing:']' Lexer.natural)
      text
  in
  assert (separated "[1,2]" = Ok [ "1"; "2" ]);
  assert (separated "[]" = Ok []);
  assert (Result.is_error (separated "[1,]"));
  assert (String_ext.words "a\t b\r\nc" = [ "a"; "b"; "c" ]);
  assert (String_ext.lines " \na\n\tb\n" = [ "a"; "b" ]);
  assert (String_ext.split_once ":=" "x:=a:=b" = Some ("x", "a:=b"));
  assert (String_ext.split_once ":=" "x" = None);
  assert (String_ext.strip_prefix "@" "@name" = "name");
  assert (String_ext.strip_suffix ";" "name;" = "name");
  assert (String_ext.strip_line_comment "//" "a // comment" = "a ");
  List.iter
    (fun text -> assert (Number.to_bytes (Number.of_bytes text) = text))
    [ ""; "Hi"; "あ"; "\000\001"; "\255" ];
  assert (Number.to_bytes "256" = "\000\001");
  assert (Number.of_string_with_plus "+001" = Ok "1");
  assert (Result.is_error (Number.of_string_with_plus "+ 1"));
  print_endline "Parsing utility tests passed"
