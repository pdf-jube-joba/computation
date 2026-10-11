open Model_registry

let get context = function
  | Ok value -> value
  | Error error -> failwith (context ^ ": " ^ error)

let roundtrip context (module T : Utils.Machine.TEXT) text =
  let value = get context (T.parse text) in
  let printed = T.print value in
  let restored = get context (T.parse printed) in
  assert (T.print restored = printed)

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
