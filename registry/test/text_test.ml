open Model_registry

let get context = function
  | Ok value -> value
  | Error error -> failwith (context ^ ": " ^ error)

let roundtrip context (module T : Utils.Machine.TEXT) text =
  let value = get context (T.parse text) in
  let printed = T.print value in
  let restored = get context (T.parse printed) in
  assert (T.print restored = printed)

let code_example = function
  | "example_counter" -> "3"
  | "lambda_calculus" -> "(\\x. x) (\\y. y)"
  | "goto_lang" -> "inc x; cpy y <- x; ifnz y : 0;"
  | "recursive_function" -> "PRIM[z:PROJ[1,0] s:COMP[SUCC:PROJ[3,0]]]"
  | "flow_ir" -> "@x 0 :main{:entry{%a:=#3;print %a;halt;}}"
  | "expr_lang" -> "x := 0 ; while x < 3 [ x := x + 1 ]"
  | "proc_lang" -> "static x;main()[local y x := 1;return]"
  | "internal_ctrl" -> "static x;loop :again (x := 1;break :again x) -> x"
  | "fn_ptr" -> "fn main(){return}"
  | "mini_prog" -> "fn main(){return 1}"
  | "pcf-expr" | "pcf-cek" -> "(fun x => inc x)(2)"
  | "pcf-expr-stmt-cek" -> "{ let x := 1; print 2; inc x }"
  | "pcf-secd" -> {|[{"PushNat":2},{"PushNat":3},{"BinOp":"Add"}]|}
  | "lambda_continuations" -> "fun x => let y = succ zero in @(x y)"
  | "logic_circuit" -> "graph: main {in {I} out {O=N.OUT} N, NOT_F {IN=I}}"
  | "turing_machine" -> "start\ngoal\na,start,b,goal,R\n"
  | "rec_tm_ir" -> "alphabet:(a,b) fn main(){label entry:{@:=const a return}}"
  | "rec_tm_ir_jump" -> "alphabet:(a,b) @:=const a RT jump 0"
  | "tiny_isa" -> "+64\n123456789012345678901234567890"
  | "symbolic_asm" -> ".text\n@main:\nldi %r0 #3;\nhalt;"
  | "cfg_vreg" -> "@main {\n%v0 := #1;\ngoto @main;\n}"
  | "strarr" -> "x := ['猫', '\\n'] ; while is_empty x { x := ['🙂'] }"
  | "strtree" -> "x := cons atom '猫' atom '🙂' ; y := left x"
  | "coroutine" | "async_await" -> "fn main { x <- 1; if x < 3 goto -1 }"
  | name -> failwith ("Missing code example for " ^ name)

let examples = function
  | "example_counter" -> ("", "inc", "dec", "End")
  | "lambda_calculus" -> ("\\x. x, \\y. y", "0", "", "\\x. x")
  | "goto_lang" -> ("x=1; y=2;", "", "", "x=3; y=4;")
  | "recursive_function" -> ("(1, 2)", "", "", "5")
  | "flow_ir" -> ("x=3\nlabel=:main\nkey=k:h[1][2]", "猫\n", "猫\n", "x=3")
  | "expr_lang" -> ("x=1\ny=2", "", "", "x=3\ny=4")
  | "proc_lang" -> ("x=1\ny=2", "猫\n", "猫\n", "x=3\ny=4")
  | "internal_ctrl" -> ("x=1\ny=2", "", "猫\n", "x=3\ny=4")
  | "fn_ptr" -> ("", "", "猫\n", "finished\n")
  | "mini_prog" -> ("(1, 2)", "", "猫\n", "5")
  | "pcf-expr" | "pcf-expr-stmt-cek" | "pcf-cek" | "pcf-secd" ->
      ("(1, 2)", "", "+002", "5")
  | "lambda_continuations" -> ("(1, 2)", "", "", "fun x => S x")
  | "logic_circuit" -> ("", "main.I=#true", "main.O=#false", "")
  | "turing_machine" | "rec_tm_ir" | "rec_tm_ir_jump" ->
      ("a,b|-|c", "", "", "a|b|c,d")
  | "tiny_isa" | "symbolic_asm" ->
      let image = "0, 1, 2, 3, 4, 5, 6, 7\n8\n9" in
      (image, "", "", image)
  | "cfg_vreg" -> ("(1, 2)", "", "", "(3, 4)")
  | "strarr" -> ("x=['猫', '\\n']", "", "", "x=['🙂']")
  | "strtree" -> ("x=cons atom '猫' atom '\\n'", "", "", "x=atom '🙂'")
  | "coroutine" | "async_await" -> ("2", "1", "猫\n", "x=1\n$task=none\n$child=2")
  | name -> failwith ("Missing text examples for " ^ name)

let () =
  List.iter
    (function
      | _, Registry.Compiler _ -> ()
      | name, Registry.Model (module M) ->
          let ainput, rinput, routput, foutput = examples name in
          roundtrip (name ^ " Code") (module M.Code) (code_example name);
          roundtrip (name ^ " Ainput") (module M.Ainput) ainput;
          roundtrip (name ^ " Rinput") (module M.Rinput) rinput;
          roundtrip (name ^ " Routput") (module M.Routput) routput;
          roundtrip (name ^ " Foutput") (module M.Foutput) foutput)
    Registry.entries;
  roundtrip "empty effect" (module Pcf.Syntax.Routput) "";
  roundtrip "numeric continuation value"
    (module Lambda_continuations.No_ext.Foutput)
    "S S 0";
  roundtrip "control continuation value"
    (module Lambda_continuations.Ctrl.Foutput)
    "fun x => abort x";
  assert (Result.is_error (Lambda_continuations.No_ext.Foutput.parse "S x"));
  assert (Result.is_error (Utils.Text.Natural.parse "-1"));
  assert (Result.is_error (Utils.Text.Natural_int.parse "-1"));
  assert (Result.is_error (Utils.Text.Unit.parse "unexpected"));
  print_endline "All model text codecs passed"
