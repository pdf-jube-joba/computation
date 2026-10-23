module M = Core_machine.Main
module T = Core_machine.Types
module N = Utils_base.Number

let get = Utils_base.Error.get
let check name b = if not b then failwith name

let error name = function
  | Error _ -> ()
  | Ok _ -> failwith (name ^ ": expected error")

let code body = "\\memories 3;\n:main {\n" ^ body ^ "\n}"

let initial src input =
  get (M.make (get (M.Code.parse src)) (get (M.Ainput.parse input)))

let rec run fuel s =
  if fuel = 0 then failwith "Step limit";
  let saved = M.Snapshot.to_yojson s in
  let restored = get (M.Snapshot.of_yojson saved) in
  check "snapshot roundtrip" (M.Snapshot.to_yojson restored = saved);
  let result = M.step s "" in
  check "step immutable" (M.Snapshot.to_yojson s = saved);
  check "restored transition" (result = M.step restored "");
  match result with
  | Error e -> Error e
  | Ok (Utils_runtime.Machine.Halt out) -> Ok (M.Foutput.print out)
  | Ok (Utils_runtime.Machine.Continue { next; _ }) -> run (fuel - 1) next

let execute body = run 200 (initial (code body) "")

let () =
  let src =
    code
      {|%s := \slice MEM_0 10 20;
%t := \slice %s 2 5;
%p := \addr %t 1;
%q := \addr %s 6;
%d := %q - %p;
%r := \addr MEM_0 0;
\deref %r := \st %d;
%x := %p - %q;
%r := %r + 1;
\deref %r := \st %x;
\if %p < %q \then :yes;
\halt;|}
    ^ ":yes { %p := %p + 1; \\deref %p := \\st 42; \\halt; }"
  in
  let parsed = get (M.Code.parse src) in
  check "code roundtrip"
    (M.Code.print parsed
    = M.Code.print (get (M.Code.parse (M.Code.print parsed))));
  check "nested slices and difference"
    (get (run 200 (get (M.make parsed []))) = "0 := 3;\n14 := 42;\n");
  check "huge sparse address and memory isolation"
    (get
       (execute
          {|%p := \addr MEM_0 1000000000000000000000000000000;
%q := \addr MEM_1 1000000000000000000000000000000;
\deref %q := \st 99;
%x := \ld \deref %p;
%x := %x + 7;
\deref %p := \st %x;
\halt;|})
    = "1000000000000000000000000000000 := 7;\n");
  List.iter
    (fun (name, body) -> error name (execute body))
    [
      ( "end load",
        {|%s := \slice MEM_0 0 1; %p := \addr %s 1; %x := \ld \deref %p; \halt;|}
      );
      ( "end store",
        {|%s := \slice MEM_0 0 1; %p := \addr %s 1; \deref %p := \st 7; \halt;|}
      );
      ( "empty slice",
        {|%s := \slice MEM_0 2 2; %p := \addr %s 0; %x := \ld \deref %p; \halt;|}
      );
      ( "advance bounds",
        {|%s := \slice MEM_0 0 1; %p := \addr %s 0; %p := %p + 2; \halt;|} );
      ("retreat underflow", {|%p := \addr MEM_0 0; %p := %p - 1; \halt;|});
      ( "different origin difference",
        {|%p := \addr MEM_0 0; %q := \addr MEM_1 0; %x := %p - %q; \halt;|} );
      ( "different origin ordering",
        {|%p := \addr MEM_0 0; %q := \addr MEM_1 0; \if %p < %q \then :main; \halt;|}
      );
      ("invalid addition", {|%p := \addr MEM_0 0; %p := 1 + %p; \halt;|});
      ("addr plus addr", {|%p := \addr MEM_0 0; %p := %p + %p; \halt;|});
      ("infinite len", {|%n := \len MEM_0; \halt;|});
      ( "nested bounds",
        {|%s := \slice MEM_0 10 20; %t := \slice %s 0 11; \halt;|} );
      ("reversed slice", {|%s := \slice MEM_0 3 2; \halt;|});
      ("invalid deref", {|\deref 0 := \st 1; \halt;|});
      ("mixed comparison", {|\if 0 == MEM_0 \then :main; \halt;|});
      ("non-code jump", {|%x := 0; \goto %x;|});
    ];
  List.iter
    (fun src -> error "invalid syntax" (M.Code.parse src))
    [
      "\\memories 0; :main { \\halt; }";
      code {|\halt; \if 0 == 0 \then :main;|};
      code {|\if 0 == 0 \then :main; %x := 1; \halt;|};
      code {|%x := 1;|};
      code {|\goto 0;|};
    ];
  List.iter
    (fun src -> error "invalid code" (M.make (get (M.Code.parse src)) []))
    [
      code {|%x := MEM_3; \halt;|};
      code {|%x := :missing; \halt;|};
      code {|\halt;|} ^ ":main { \\halt; }";
      "\\memories 1; :other { \\halt; }";
    ];
  let src =
    code
      {|%a := \slice MEM_0 10 20; %b := \slice MEM_0 12 15;
%p := \addr %a 3; %q := \addr %b 1;
\if %p == %q \then :equal; \halt;|}
    ^ ":equal { %p := \\addr MEM_0 0; \\deref %p := \\st 1; \\halt; }"
  in
  check "same location different bounds"
    (get (run 200 (initial src "")) = "0 := 1;\n");
  check "different origin equality false"
    (get
       (execute
          {|%p := \addr MEM_0 0; %q := \addr MEM_1 0; \if %p == %q \then :main; \halt;|})
    = "");
  let src =
    code
      {|%f := :done; %p := \addr MEM_0 0; \deref %p := \st %f;
%f := \ld \deref %p; \goto %f;|}
    ^ ":done { \\halt; }"
  in
  check "code value indirect jump"
    (get (run 200 (initial src "")) = "0 := :done;\n");
  let input =
    "0 := \\addr \\slice MEM_0 2 5 1;\n1 := \\slice MEM_1 0 4;\n2 := :main;\n"
  in
  let parsed = get (M.Ainput.parse input) in
  check "input roundtrip" (get (M.Ainput.parse (M.Ainput.print parsed)) = parsed);
  check "input output values"
    (get (run 200 (initial (code {|\halt;|}) input)) = input);
  error "duplicate input"
    (M.make
       (get (M.Code.parse (code {|\halt;|})))
       (get (M.Ainput.parse "0 := 1; 0 := 2;")));
  error "invalid input address"
    (M.make
       (get (M.Code.parse (code {|\halt;|})))
       (get (M.Ainput.parse "0 := \\addr \\slice MEM_0 0 1 2;")));
  let s = initial (code {|\halt;|}) "" in
  error "snapshot line"
    (M.Snapshot.of_yojson (T.state_to_yojson { s with line = 99 }));
  error "snapshot memory count"
    (M.Snapshot.of_yojson (T.state_to_yojson { s with memory = [||] }));
  let invalid_memory = Array.copy s.memory in
  invalid_memory.(0) <-
    [ (N.zero, T.Address (T.Subslice (T.Whole 0, N.zero, N.one), N.of_int 2)) ];
  error "snapshot bounds"
    (M.Snapshot.of_yojson
       (T.state_to_yojson { s with memory = invalid_memory }));
  print_endline "core_machine tests passed"
