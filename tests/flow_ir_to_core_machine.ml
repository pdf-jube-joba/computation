module F = Flow_ir.Main
module C = Core_machine.Main
module Compiler = Flow_ir_to_core_machine.Main
module Machine = Utils_runtime.Machine

let get = Utils_base.Error.get
let check name b = if not b then failwith name
let trace out = List.filter (fun s -> s <> "") (List.rev out)

let rec run_source fuel s input out =
  if fuel = 0 then failwith "source fuel";
  match F.step s input with
  | Error _ -> Error (trace out)
  | Ok (Machine.Halt v) -> Ok (v, trace out)
  | Ok (Machine.Continue { next; output }) ->
      run_source (fuel - 1) next input (output :: out)

let rec run_target fuel s input out =
  if fuel = 0 then failwith "target fuel";
  match C.step s input with
  | Error _ -> Error (trace out)
  | Ok (Machine.Halt v) -> Ok (get (Compiler.decode_foutput v), trace out)
  | Ok (Machine.Continue { next; output }) ->
      run_target (fuel - 1) next input
        (get (Compiler.decode_routput output) :: out)

let compare name code input rinput =
  let source = get (F.Code.parse code) in
  let input = get (F.Ainput.parse input) in
  let compiled = get (Compiler.compile source) in
  let printed = C.Code.print compiled in
  check
    (name ^ " target code roundtrip")
    (C.Code.print (get (C.Code.parse printed)) = printed);
  let actual =
    run_target 200000
      (get (C.make compiled (get (Compiler.encode_ainput input))))
      (get (Compiler.encode_rinput rinput))
      []
  in
  let expected =
    match F.make source input with
    | Error _ -> Error []
    | Ok s -> run_source 10000 s rinput []
  in
  check name (actual = expected)

let program body = "@x 0 @y 7 :main { :entry { " ^ body ^ " } }"

let () =
  List.iter
    (fun (name, body, input, rinput) ->
      compare name (program body) input rinput)
    [
      ( "arithmetic",
        {|%a := \ld @y; %a := %a + 4; @x := \st %a; \halt;|},
        "",
        "" );
      ("input overrides", {|\halt;|}, "x=8\ny=9", "");
      ("empty input", {|\halt;|}, "", "");
      ( "register alias",
        {|%a := 12; %a := %a - %a; @x := \st %a; \halt;|},
        "",
        "" );
      ( "static pointer",
        {|%p := \ref @x; %q := \ref \deref %p; \deref %q := \st 12; @y := \st %q; \halt;|},
        "",
        "" );
      ( "stack",
        {|\push 10; \push 20; \lget %n; %p := \ref \sacc 0; \deref %p := \st %n; \pop %v; @x := \st %v; %v := \ld \sacc 0; @y := \st %v; \halt;|},
        "",
        "" );
      ( "stack popped key",
        {|\push 1; %p := \ref \sacc 0; \pop %v; %v := \ld \deref %p; \halt;|},
        "",
        "" );
      ( "stack ref revived",
        {|\push 1; %p := \ref \sacc 0; \pop %v; \push 5; %v := \ld \deref %p; @x := \st %v; \halt;|},
        "",
        "" );
      ("empty pop", {|\pop %v; \halt;|}, "", "");
      ( "heap",
        {|\halloc(2) %h; %p := \ref \hacc(%h)[1]; \deref %p := \st 42; %v := \ld \deref %p; @x := \st %v; @y := \st %p; \halt;|},
        "",
        "" );
      ( "heap zero",
        {|\halloc(1) %h; %v := \ld \hacc(%h)[0]; @y := \st %v; \halt;|},
        "",
        "" );
      ( "heap reallocation",
        {|\halloc(1) %h; \hacc(%h)[0] := \st 99; \hfree %h; \halloc(1) %g; %v := \ld \hacc(%g)[0]; @x := \st %g; @y := \st %v; \halt;|},
        "",
        "" );
      ( "use after free",
        {|\halloc(1) %h; %p := \ref \hacc(%h)[0]; \hfree %h; %v := \ld \deref %p; \halt;|},
        "",
        "" );
      ("heap bounds", {|\halloc(1) %h; %v := \ld \hacc(%h)[1]; \halt;|}, "", "");
      ( "zero allocation",
        {|\halloc(0) %h; \hfree %h; @x := \st %h; \halt;|},
        "",
        "" );
      ("double free", {|\halloc(1) %h; \hfree %h; \hfree %h; \halt;|}, "", "");
      ( "numeric handle",
        {|\halloc(1) %h; \hacc(1)[0] := \st 8; %v := \ld \hacc(%h)[0]; @x := \st %v; \halt;|},
        "",
        "" );
      ( "dangling ref permitted",
        {|%p := \ref \hacc(99)[3]; @x := \st %p; \halt;|},
        "",
        "" );
      ("bad arithmetic", {|%p := \ref @x; %p := %p + 1; \halt;|}, "", "");
      ("bad deref", {|%p := 0; \deref %p := \st 1; \halt;|}, "", "");
      ( "bad ordered keys",
        {|%p := \ref @x; \if %p < %p \then :entry; \halt;|},
        "",
        "" );
      ( "mixed equality",
        {|%p := \ref @x; \if %p = 0 \then :entry; \halt;|},
        "",
        "" );
      ( "input references",
        {|%p := \ld @x; \deref %p := \st 88; \halt;|},
        "x=\\static @y",
        "" );
      ("bad input static", {|\halt;|}, "x=\\static @missing", "");
      ("unknown input", {|\halt;|}, "unknown=1", "");
      ("io", {|\input @x; %v := \ld @x; \print %v; \halt;|}, "", "hello");
      ("invalid utf8", {|%v := 255; \print %v; \halt;|}, "", "");
    ];

  compare "regions reset and indirect enter"
    {|@x 0 @y 9 :main {
 :entry { %flag := \ld @x; \if %flag = 1 \then :done; \goto :call; }
 :call { %r := :main; \push %r; %v := 99; \enter :worker; }
 :done { @y := \st %v; \halt; }
 } :worker { :entry { @x := \st 1; \pop %r; \enter %r; } }|}
    "" "";
  compare "local label shadowing"
    {|@x 0 :main {
 :entry { \goto :main; } :main { @x := \st 5; \halt; }
 }|}
    "" "";
  compare "computed local jump and code output"
    {|@x 0 :main {
 :entry { %r := :done; @x := \st %r; \goto %r; } :done { \halt; }
 }|}
    "" "";
  compare "code input resolution"
    {|@x 0 :main {
 :entry { %r := \ld @x; \goto %r; } :done { @x := \st %r; \halt; }
 }|}
    "x=:done" "";
  compare "cross region goto rejected"
    {|:main { :entry { %r := :other; \goto %r; } }
 :other { :start { \halt; } }|}
    "" "";
  compare "block cannot enter"
    {|:main { :entry { %r := :done; \enter %r; } :done { \halt; } }|} "" "";
  compare "zero statics" {|:main { :entry { \halt; } }|} "" "";

  compare "loop"
    {|@x 0 :main {
    :entry { %n := 5; %sum := 0; \goto :loop; }
    :loop { %sum := %sum + %n; %n := %n - 1; \if %n > 0 \then :loop; \goto :done; }
    :done { @x := \st %sum; \halt; }
  }|}
    "" "";
  compare "io heap and stack"
    (program
       {|\halloc(1) %h; \input \hacc(%h)[0];
    %v := \ld \hacc(%h)[0]; \print %v; \push 0; \input \sacc 0;
    \pop %v; \print %v; @x := \st %v; \halt;|})
    "" "日本語";
  compare "invalid code input" (program {|\halt;|}) "x=:missing" "";
  compare "input heap and stack references" (program {|\halt;|})
    "x=\\heap[42][3]\ny=\\stack[5]" "";
  compare "large numeric payload"
    (program {|%x := \ld @x; %x := %x + 1; @x := \st %x; \halt;|})
    "x=9999999999999999999999999999999999999" "";
  print_endline "flow_ir_to_core_machine differential tests passed"
