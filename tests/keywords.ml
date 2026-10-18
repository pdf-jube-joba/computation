module Registry = Model_registry.Registry
module Text = Utils_data.Text
module Session = Utils_runtime.Session

let get = Utils_base.Error.get
let check message condition = if not condition then failwith message

let roundtrip (module Codec : Text.S) text =
  let printed = Codec.print (get (Codec.parse text)) in
  check
    ("codec roundtrip: " ^ text)
    (Codec.print (get (Codec.parse printed)) = printed)

let code name text =
  let module M = (val get (Registry.model name)) in
  roundtrip (module M.Code) text

let reject name text =
  let module M = (val get (Registry.model name)) in
  check
    (name ^ " accepted legacy syntax: " ^ text)
    (Result.is_error (M.Code.parse text))

let run name text ainput rinput expected =
  let module M = (val get (Registry.model name)) in
  let module S = Session.Make (M) in
  let session = S.create () in
  get (S.make session text ainput);
  let rec finish remaining =
    if remaining = 0 then failwith (name ^ ": step limit");
    match get (S.step session rinput) with
    | Session.Continued _ -> finish (remaining - 1)
    | Session.Halted text -> text
  in
  check (name ^ ": unexpected output") (finish 2000 = expected)

let identifiers () =
  run "lambda_calculus" {|(\fun fun. fun) if|} "" "0" "if";
  run "pcf-expr" {|(\fun inc => inc)(3)|} "()" "" "3";
  run "pcf-cek" {|(\fun if => \inc if)(3)|} "()" "" "4";
  run "pcf-expr" {|\if \not \false \then 2 \else 0 \fi|} "()" "" "2";
  run "expr_lang" "if := 3; then := if" "" "" "if = 3\nthen = 3\n";
  run "expr_lang" {|x := \if if = 0 \then 2 \else 3|} "if = 0" ""
    "if = 0\nx = 2\n";
  run "goto_lang" {|\inc inc;|} "" "" "inc = 1";
  run "recursive_function" {|\let SUCC = \ZERO. SUCC|} "()" "" "0";
  run "recursive_function" {|\let SUCC = \ZERO. \SUCC|} "(3)" "" "4";
  run "flow_ir"
    {|@halt 0 :main { :entry { %ld := #5; @halt := \st %ld; \halt; } }|} "" ""
    "halt = 5\n";
  run "rec_tm_ir"
    {|\alphabet: (-, const) \fn main { \label return: { const := \const const @ := const \return } }|}
    "|-|" "" "|const|";
  code "strarr" {|tail := ['a']; head := \tail tail|};
  code "strtree"
    {|atom := \atom 'a'; cons := \cons atom atom; left := \left cons|};
  code "fn_ptr" {|\fn main(return) { \assign return := \null_ptr; \return }|};
  code "fn_ptr" {|\fn main(x) { \assign (x \addr) \loc := \ld x; \return }|};
  code "mini_prog"
    {|\fn main(if: \bool) { \if \ld \local if \return 1; \return 0 }|};
  code "lambda_continuations" {|\fun succ => succ|};
  roundtrip
    (module Lambda_continuations_shared.Syntax.Nat_code)
    {|\fun fun => fun|}

let values () =
  roundtrip (module Text.Bool) {|\true|};
  roundtrip (module Utils_data.Tape.Direction) {|\L|};
  roundtrip (module Utils_data.Character_tree) {|\cons \atom 'a' \atom 'b'|};
  roundtrip (module Fn_ptr.Runtime_value) {|\loc(0, 1)|};
  roundtrip (module Fn_ptr.Runtime_value) {|\null_ptr|};
  List.iter
    (fun text ->
      let value = get (Flow_ir.Input_text.parse_value text) in
      let printed = Flow_ir.Input_text.print_value value in
      check
        ("Flow IR value codec: " ^ text)
        (get (Flow_ir.Input_text.parse_value printed) = value))
    [ {|\static @if|}; {|\stack[1]|}; {|\heap[2][3]|}; ":if"; "3" ];
  check "bare boolean rejected" (Result.is_error (Text.Bool.parse "true"));
  check "legacy boolean rejected" (Result.is_error (Text.Bool.parse "#true"));
  check "legacy tree constructors rejected"
    (Result.is_error (Utils_data.Character_tree.parse "atom 'a'"));
  check "legacy direction rejected"
    (Result.is_error (Utils_data.Tape.Direction.parse "L"));
  check "strings preserve words and backslashes"
    (get (Text.String.parse {|if \if #true|}) = {|if \if #true|});
  check "HDL boolean literal"
    (get (My_hdl.Main.Parser.parse_value {|\true|}) = My_hdl.Main.Value.Bit true)

let embedded_builtins () =
  run "symbolic_asm"
    {|\equ CODE_LEN 7
\ldi %r1 CODE_LEN
\ldi %r2 \CODE_LEN
\halt|}
    "0,0,0,0,0,0,0,0" "" "2, 7, 3, 0, 0, 0, 0, 0";
  code "symbolic_asm" {|\text
@THIS:
\ldi \f %r1 \THIS
\data
\DATA_LEN|};
  code "logic_circuit"
    {|\graph: NOT_F { \in {I} \out {O=N.OUT} N, \NOT_F {IN=I} }
\graph: main { \in {I} \out {O=A.O} A, NOT_F {I=I} }|};
  code "logic_circuit"
    {|\graph: main { \in {I} \out {O=A.OUT} A, \NOT_F {IN=I} }|};
  reject "logic_circuit"
    {|\graph: main { \in {I} \out {O=A.OUT} A, NOT_F {IN=I} }|};
  reject "lambda_calculus" {|\x. x|};
  reject "pcf-expr" "fun x => x";
  reject "pcf-expr" {|\fun \if => 3|};
  reject "goto_lang" "inc x;";
  reject "strarr" "x := tail x";
  reject "symbolic_asm" "ldi %r1 3";
  reject "symbolic_asm" {|\ldi.f %r1 3|}

let () =
  identifiers ();
  values ();
  embedded_builtins ();
  print_endline "Passed keyword, identifier and codec checks."
