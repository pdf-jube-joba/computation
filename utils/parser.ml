(* Small syntax helpers over the shared token stream. Grammar stays in models. *)
type 'a t = Lexer.t -> 'a

let run ?(create = fun text -> Lexer.create text) parse text =
  Lexer.protect (fun () ->
      let p = create text in
      let value = parse p in
      Lexer.expect_eof p;
      value)

let between opening closing parse p =
  Lexer.expect_symbol p opening;
  let value = parse p in
  Lexer.expect_symbol p closing;
  value

(* Use after a grammar has already consumed the opening delimiter. *)
let closing delimiter parse p =
  let value = parse p in
  Lexer.expect_symbol p delimiter;
  value

let parens parse = between '(' ')' parse
let brackets parse = between '[' ']' parse
let braces parse = between '{' '}' parse
