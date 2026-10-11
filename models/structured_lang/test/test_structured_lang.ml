module S = Structured_lang
module E = S.Expr_lang
module C = S.Internal_ctrl
module P = S.Proc_lang
module F = S.Fn_ptr
module M = S.Mini_prog
module I = Flow_ir

let get = function Ok x -> x | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "Expected error"
let eq a b = if a <> b then failwith ("Expected " ^ b ^ ", got " ^ a)

module Run (M : Utils.Machine.S) = struct
  let run code input rinput =
    let rec loop fuel s =
      if fuel = 0 then failwith "Step limit";
      let snap = M.snapshot s in
      let resumed = get (M.restore snap) in
      if M.snapshot resumed <> snap then failwith "Snapshot mismatch";
      ignore (M.render resumed);
      if M.step s rinput <> M.step resumed rinput then
        failwith "Resume changes step";
      match get (M.step resumed rinput) with
      | Utils.Machine.Halt output -> output
      | Continue { next; _ } -> loop (fuel - 1) next
    in
    loop 10000 (get (M.make code input))
end

module RE = Run (E.Machine)
module RC = Run (C.Machine)
module RP = Run (P.Machine)
module RF = Run (F.Machine)
module RM = Run (M.Machine)
module RI = Run (I.Machine)
module Expr_compiler_contract : Utils.Compiler.S = S.Expr_lang_compiler
module Ctrl_compiler_contract : Utils.Compiler.S = S.Internal_ctrl_compiler
module Proc_compiler_contract : Utils.Compiler.S = S.Proc_lang_compiler

let contains s part =
  let len = String.length part in
  let rec loop i =
    i + len <= String.length s && (String.sub s i len = part || loop (i + 1))
  in
  loop 0

let test_expr () =
  let source =
    {|while x < 3 [ if !(x = 1) y := if x < 2 then x + 1 else x - 1 ; x := x + 1 ]|}
  in
  let code = get (E.parse_code source) in
  if get (E.parse_code (E.print code)) <> code then failwith "Expr roundtrip";
  let out =
    RE.run (get (E.parse_code "x := 0 ; while x < 3 [ x := x + 1 ]")) [] ()
  in
  eq (List.assoc "x" out) "3";
  let out =
    RE.run
      (get (E.parse_code "x := 1 - 3 ; y := if x = 0 || x > 3 then 8 else 9"))
      [] ()
  in
  eq (List.assoc "x" out) "0";
  eq (List.assoc "y" out) "8";
  let s = get (E.make (get (E.parse_code "x := 4 ; x := x + 1")) []) in
  let s =
    match get (E.step s ()) with
    | Continue { next; _ } -> next
    | _ -> failwith "early halt"
  in
  if
    s.stmt
    <> E.Seq
         (E.Nop, E.Assign ("x", E.BinOp (E.Var "x", S.Common.Add, E.Num "1")))
  then failwith "Expr small step";
  let source =
    "x := 0 ; while x < 3 [ if x < 2 y := if x = 1 then 12 else 5 ; x := x + 1 \
     ]"
  in
  let code = get (E.parse_code source) in
  let expected = RE.run code [] () in
  let compiled = get (S.Expr_lang_compiler.compile code) in
  if get (I.parse_code (I.print compiled)) <> compiled then
    failwith "Expr IR roundtrip";
  let actual =
    get (S.Expr_lang_compiler.decode_foutput (RI.run compiled [] ""))
  in
  eq (List.assoc "x" actual) (List.assoc "x" expected);
  eq (List.assoc "y" actual) (List.assoc "y" expected);
  ignore (get (S.Expr_lang_compiler.api.compile_code source));
  eq (get (S.Expr_lang_compiler.api.encode_ainput "x = 2")) "x = 2\n"

let test_ctrl () =
  let source =
    "static x, y; loop :outer ( if x < 3 { x := x + 1; continue :outer }; \
     break :outer x ) -> y"
  in
  let code = get (C.parse_code source) in
  if get (C.parse_code (C.print code)) <> code then failwith "Ctrl roundtrip";
  let expected = RC.run code [ ("x", "0"); ("y", "0") ] () in
  eq (List.assoc "x" expected) "3";
  eq (List.assoc "y" expected) "3";
  let ir = get (S.Internal_ctrl_compiler.compile code) in
  if get (I.parse_code (I.print ir)) <> ir then failwith "Ctrl IR roundtrip";
  let actual =
    get
      (S.Internal_ctrl_compiler.decode_foutput
         (RI.run ir
            (get
               (S.Internal_ctrl_compiler.encode_ainput
                  [ ("x", "0"); ("y", "0") ]))
            ""))
  in
  if actual <> S.Common.sorted expected then failwith "Ctrl compiler mismatch";
  let source =
    "static x, y, z; loop :outer ( loop :inner ( x := 17; break :outer x ) -> \
     z ) -> y"
  in
  let code = get (C.parse_code source) in
  let expected = RC.run code [ ("x", "0"); ("y", "0"); ("z", "4") ] () in
  eq (List.assoc "y" expected) "17";
  eq (List.assoc "z" expected) "4";
  let actual =
    get
      (S.Internal_ctrl_compiler.decode_foutput
         (RI.run
            (get (S.Internal_ctrl_compiler.compile code))
            (get
               (S.Internal_ctrl_compiler.encode_ainput
                  [ ("x", "0"); ("y", "0"); ("z", "4") ]))
            ""))
  in
  if actual <> S.Common.sorted expected then
    failwith "Ctrl nested compiler mismatch";
  error (C.make code []);
  error (C.make code [ ("unknown", "0") ]);
  error
    (S.Internal_ctrl_compiler.compile
       (get (C.parse_code "static;break :bad x")));
  let s = get (C.make (get (C.parse_code "static;continue :bad")) []) in
  let s =
    match get (C.step s ()) with
    | Continue { next; _ } -> next
    | _ -> failwith "early halt"
  in
  error (C.step s ())

let test_proc () =
  let source =
    {|static out;
inc(x)[local y y := x + 1;return y]
main()[local a, b a := 5;call inc(a) -> b;out := b;return]|}
  in
  let code = get (P.parse_code source) in
  if get (P.parse_code (P.print code)) <> code then failwith "Proc roundtrip";
  let expected = RP.run code [ ("out", "0") ] "" in
  eq (List.assoc "out" expected) "6";
  let actual =
    get
      (S.Proc_lang_compiler.decode_foutput
         (RI.run
            (get (S.Proc_lang_compiler.compile code))
            (get (S.Proc_lang_compiler.encode_ainput [ ("out", "0") ]))
            ""))
  in
  if actual <> expected then failwith "Proc compiler mismatch";
  let source =
    {|static out;
f(x, y)[local z z := x + y;return z, y]
main()[local a, b, c a := 3;b := 4;call f(a, b) -> a, c;out := a + c;return]|}
  in
  let code = get (P.parse_code source) in
  let expected = RP.run code [ ("out", "0") ] "" in
  eq (List.assoc "out" expected) "11";
  let ir = get (S.Proc_lang_compiler.compile code) in
  if get (I.parse_code (I.print ir)) <> ir then failwith "Proc IR roundtrip";
  let actual =
    get
      (S.Proc_lang_compiler.decode_foutput
         (RI.run ir [ ("out", I.Num "0") ] ""))
  in
  if actual <> expected then failwith "Multiarg/return compiler mismatch";
  let source =
    {|static out;
f(x)[local y y := x;if x > 0 [y := x - 1;call f(y) -> y];return y]
main()[local a a := 5;call f(a) -> a;out := a;return]|}
  in
  let code = get (P.parse_code source) in
  eq (List.assoc "out" (RP.run code [ ("out", "0") ] "")) "0";
  let actual =
    get
      (S.Proc_lang_compiler.decode_foutput
         (RI.run
            (get (S.Proc_lang_compiler.compile code))
            [ ("out", I.Num "0") ]
            ""))
  in
  eq (List.assoc "out" actual) "0";
  let source =
    "static out;main()[local a a := 0;while a < 4 [a := a + 1];out := a]"
  in
  let code = get (P.parse_code source) in
  eq (List.assoc "out" (RP.run code [ ("out", "0") ] "")) "4";
  eq
    (List.assoc "out"
       (get
          (S.Proc_lang_compiler.decode_foutput
             (RI.run
                (get (S.Proc_lang_compiler.compile code))
                [ ("out", I.Num "0") ]
                ""))))
    "4";
  error (P.make code []);
  error
    (P.make
       (get (P.parse_code "static x;main()[local x return]"))
       [ ("x", "0") ]);
  error (P.make (get (P.parse_code "static;main(x)[local y return]")) [])

let test_fnptr () =
  List.iter
    (fun text ->
      let value = get (F.Runtime_value.parse text) in
      if get (F.Runtime_value.parse (F.Runtime_value.print value)) <> value then
        failwith "FnPtr value roundtrip")
    [ "-7"; "#null_ptr"; "loc(3, 2)" ];
  let source =
    {|fn inc(ptr) {assign (ld ptr) #loc := (ld (ld ptr) #loc) + 1;return}
 fn main(x) {assign x := 4;call inc(x #addr);return}|}
  in
  let code = get (F.parse_code source) in
  if get (F.parse_code (F.print code)) <> code then failwith "FnPtr roundtrip";
  let output = RF.run code () () in
  if not (contains output "loc(0, 0) = 5") then failwith output;
  let output =
    RF.run
      (get
         (F.parse_code
            "fn main(x){assign x := 0 - 1; ifz ld x then assign x := 9 else \
             assign x := 8 end; return}"))
      () ()
  in
  if not (contains output "= 8") then failwith output;
  let output =
    RF.run
      (get
         (F.parse_code
            "fn main(x){ifz ld x then assign x := 7 else assign x := 8 \
             end;return}"))
      () ()
  in
  if not (contains output "= 7") then failwith output;
  error (F.make (get (F.parse_code "fn main(){ }")) ());
  let s =
    get
      (F.make
         (get (F.parse_code "fn main(x){assign (ld x) #loc := 1;return}"))
         ())
  in
  error (F.step s ());
  let source =
    {|fn save(p){assign (ld p) #loc := p #addr;return}
fn main(x){call save(x #addr);assign x := ld (ld x) #loc;return}|}
  in
  let code = get (F.parse_code source) in
  let rec go fuel s =
    if fuel = 0 then failwith "Expected dangling pointer error"
    else
      match F.step s () with
      | Error _ -> ()
      | Ok (Halt _) -> failwith "Dangling pointer succeeded"
      | Ok (Continue { next; _ }) -> go (fuel - 1) next
  in
  go 100 (get (F.make code ()))

let test_mini () =
  let run src input = RM.run (get (M.parse_code src)) input () in
  eq (run "fn main(x: #num){return ld local x}" [ "7" ]) "7";
  eq
    (run
       {|fn pair_up(x: #num,y: #num){return ld local x,ld local y}
 fn main(a: #num,b: #num){block(x: #num,y: #num){call fn pair_up(ld local a,ld local b) -> local x,local y;return ld local x ld local y +}}|}
       [ "3"; "4" ])
    "7";
  eq
    (run
       {|fn main(x: #num){block(p: #ptr){halloc #num -> local p;assign (ld local p) #loc := ld local x;block(x: #num){assign local x := 0;return ld (ld local p) #loc}}}|}
       [ "9" ])
    "9";
  eq
    (run
       {|static arr:[#num;3]
 static pair:#num * #bool
 static sum:#num + #bool
 fn main(){assign static arr[1] := 7;assign static pair := 9 #true pair(2);assign static pair.0 := ld static arr[1];assign static sum := #true tag(1);case 1 of ld static sum assign static sum?1 := #false;if ld static pair.1 return ld static pair.0;return 0}|}
       [])
    "7";
  eq
    (run
       {|fn main(){block(p:#ptr,x:#num){halloc #num -> local p;assign (ld local p) #loc := 11;assign local x := ld (ld local p) #loc;hfree ld local p;return ld local x}}|}
       [])
    "11";
  eq
    (run
       {|fn main(x:#num){loop again: block(){if ld local x 3 < block(){assign local x := ld local x 1 +;continue again};break again};return ld local x}|}
       [ "0" ])
    "3";
  eq (run {|fn main(){if #false ! 'a' 'a' == && return 5;return 0}|} []) "5";
  eq (run {|fn main(){if 'あ' 'あ' == return 6;return 0}|} []) "6";
  let source =
    {|fn add(x:#num){return ld local x 1 +}
 fn main(){block(f:#fn,y:#num){assign local f := fn add;call ld local f(8) -> local y;return ld local y}}|}
  in
  eq (run source []) "9";
  eq
    (run "fn main(x:#num){return ld local x}" (get (M.Ainput.parse "(7,)")))
    "7";
  eq
    (run
       {|static u:#unit static p:#ptr static f:#fn
 fn main(){if ld static u #unit == ld static p #null_ptr == && ld static f #null_fn == && return 1;return 0}|}
       [])
    "1";
  eq
    (run
       {|static s:#num + #bool fn main(){case 1 of ld static s return 99;return ld static s?0}|}
       [])
    "0";
  eq
    (run {|fn main(){loop outer: loop inner: continue outer;return 13}|} [])
    "13";
  eq
    (run
       {|fn main(){block(x:#u8){assign local x := 255;block(x:#num){assign local x := 2};return ld local x}}|}
       [])
    "255";
  eq
    (run
       {|fn bump(p:#ptr){assign (ld local p) #loc := ld (ld local p) #loc 1 +;return 0}
 fn main(x:#num){block(y:#num){call fn bump(local x #addr) -> local y;return ld local x}}|}
       [ "19" ])
    "20";
  eq
    (run
       {|fn rec(x:#num){if ld local x 0 == return 0;block(y:#num){call fn rec(ld local x 1 -) -> local y;return ld local y 1 +}}
 fn main(x:#num){block(y:#num){call fn rec(ld local x) -> local y;return ld local y}}|}
       [ "5" ])
    "5";
  let source =
    "fn main(){block(p:#ptr){assign local p := #null_ptr;return ld (ld local \
     p) #loc}}"
  in
  let rec expect_runtime_error fuel s =
    if fuel = 0 then failwith "No runtime error"
    else
      match M.step s () with
      | Error _ -> ()
      | Ok (Halt _) -> failwith "Expected runtime error"
      | Ok (Continue { next; _ }) -> expect_runtime_error (fuel - 1) next
  in
  List.iter
    (fun source ->
      expect_runtime_error 100 (get (M.make (get (M.parse_code source)) [])))
    [
      source;
      "fn main(){return 0 1 -}";
      "static x:#u8 fn main(){assign static x := 256;return 0}";
      "fn main(){return #true}";
      "fn main(){return 1,2}";
      "fn main(){if 1 return 0;return 0}";
      "fn main(){case 0 of #true return 0;return 0}";
      "static s:#num + #bool fn main(){return ld static s?1}";
      "fn main(){block(f:#fn,y:#num){call ld local f() -> local y;return 0}}";
      "static p:#num * #num fn main(){return ld static p.2}";
      "static x:[#num;1] fn main(){return ld static x[1]}";
      "fn main(){block(p:#ptr){halloc #num -> local p;hfree ld local p;return \
       ld (ld local p) #loc}}";
    ];
  error (M.make (get (M.parse_code "fn main(x:#bool){return 0}")) [ "1" ]);
  error (M.parse_code "fn main(){return 1 +}");
  let code = get (M.parse_code source) in
  if get (M.parse_code (M.print code)) <> code then failwith "Mini roundtrip"

let () =
  test_expr ();
  test_ctrl ();
  test_proc ();
  test_fnptr ();
  test_mini ();
  print_endline "structured_lang tests passed"
