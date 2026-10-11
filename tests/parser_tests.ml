open Utils

let get = function Ok x -> x | Error message -> failwith message
let bad = function Error _ -> () | Ok _ -> failwith "Expected parse error"

let () =
  let p = Lexer.create {|while \while \is-empty while_more|} in
  assert (Lexer.identifier p = "while");
  assert (not (Lexer.eat_keyword p "while_more"));
  assert (Lexer.eat_keyword p "while");
  assert (Lexer.keyword p = "is-empty");
  assert (Lexer.identifier p = "while_more");
  assert (Lexer.eof p);
  bad (Lexer.parse Lexer.identifier {|\while|});
  bad (Lexer.parse (fun p -> Lexer.expect_keyword p "while") "while");
  bad (Lexer.parse Lexer.keyword "\\");
  bad (Lexer.parse Lexer.keyword "\\ while");
  let p = Lexer.create {|"// /* literal */" '/' '猫' '\''|} in
  assert (Lexer.string_literal p = "// /* literal */");
  assert (Lexer.character p = "/");
  assert (Lexer.character p = "猫");
  assert (Lexer.character p = "'");
  assert (Lexer.eof p);
  let text = "/* outer /* nested */ done */\n// line\n{[(42)]}" in
  let parse = Parser.braces (Parser.brackets (Parser.parens Lexer.integer)) in
  assert (get (Parser.run parse text) = 42);
  bad (Parser.run parse "{[(42)]");
  bad (Parser.run parse "{[(42)]} extra");
  bad (Parser.run parse "{[(42])}");
  bad (Parser.run parse "/* outer /* inner */ {[(42)]}");
  assert (
    Lexer.parse Lexer.identifier "a/*comment*/b"
    = Error "Unexpected trailing tokens at byte 12");
  assert (
    Lexer.parse Lexer.identifier "/* comment */ \\while"
    = Error "Expected identifier at byte 14");
  bad (Lexer.parse Lexer.character "'\192\128'");
  bad (Lexer.parse Lexer.character "'\237\160\128'");
  bad (Lexer.parse Lexer.character "'ab'");
  let create text = Lexer.create ~identifier_extra:(( = ) '-') text in
  assert (get (Parser.run ~create Lexer.identifier "while-name") = "while-name");
  assert (get (Bool.parse "/* nested /* comment */ */ \\true") = Bool.T);
  bad (Bool.parse "true");
  bad (Bool.parse "#true");
  assert (Machine.empty_input "// nothing\n/* to read */" = Ok ());
  bad (Machine.empty_input "/* unclosed");
  let code =
    get
      (Goto_lang.parse_code
         {|\inc while; /* outer /* inner */ */ \cpy if <- while;|})
  in
  assert (Goto_lang.parse_code (Goto_lang.print_code code) = Ok code);
  bad (Goto_lang.parse_code "inc while;");
  assert (
    get (Lambda_calculus.parse_code "/* nested /* c */ */ \\while. while")
    |> Lambda_calculus.print = "\\while. while");
  print_endline
    "Shared lexer, keyword namespace, comments and delimiters tests passed"
