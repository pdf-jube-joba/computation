open Recursive_function_turing_machine
module I = Rec_tm_ir
module J = Rec_tm_ir_jump
module C = Rec_to_ir
module T = Turing_machine
module R = Recursive_function
module _ : Utils.Machine.S = T.Machine
module _ : Utils.Machine.S = I.Machine
module _ : Utils.Machine.S = J.Machine
module _ : Utils.Compiler.S = Compile_jump
module _ : Utils.Compiler.S = Compile_tm
module _ : Utils.Compiler.S = C
module _ : Utils.Compiler.S = Rec_to_tm

let get = function Ok v -> v | Error e -> failwith e
let check s b = if not b then failwith s
let tape s = get (T.parse_tape s)

let run step s =
  let rec loop n s =
    if n = 0 then failwith "step limit exceeded"
    else
      match get (step s ()) with
      | Utils.Machine.Halt t -> t
      | Utils.Machine.Continue { next; _ } -> loop (n - 1) next
  in
  loop 1000000 s

let run_ir p t = run I.step (get (I.make p t))
let run_jump p t = run J.step (get (J.make p t))
let run_tm p t = run T.step (get (T.make p t))

let equivalent text input expected =
  let p = get (I.parse_code text) in
  let t = tape input in
  let output = run_ir p t in
  check "IR result" (T.tape_equal output (tape expected));
  let jump = get (Compile_jump.compile p) in
  check "jump roundtrip" (get (J.parse_code (J.print jump)) = jump);
  check "IR to jump equivalence" (T.tape_equal output (run_jump jump t));
  let tm = get (Compile_tm.compile jump) in
  check "Turing roundtrip" (get (T.parse_code (T.print_code tm)) = tm);
  check "jump to Turing equivalence" (T.tape_equal output (run_tm tm t))

let helper make input expected =
  let ctx = C.create () in
  let f = make ctx in
  let p = C.wrap_function ctx f in
  let input = tape input and expected = tape expected in
  check ("helper " ^ f.I.name) (T.tape_equal (run_ir p input) expected)

let recursive f input expected =
  let t = get (C.write_usize input) in
  let p = get (C.compile f) in
  check "compiled text roundtrip" (get (I.parse_code (I.print p)) = p);
  let output = run_ir p t in
  check
    ("recursive result: " ^ R.print f)
    (C.read_right_one_usize output = Some [ expected ]);
  let jump = get (Compile_jump.compile p) in
  check "recursive IR-jump equivalence" (T.tape_equal output (run_jump jump t));
  let tm = get (Compile_tm.compile jump) in
  check "recursive jump-Turing equivalence" (T.tape_equal output (run_tm tm t))

let () =
  equivalent
    "alphabet:(a,b) fn f { label entry:{ x:=@ } } fn main {label entry:{x:=@ \
     RT call f LT @:=x}}"
    "-|a|b" "-|a|b";
  equivalent
    "alphabet:(a,b) fn g {label entry:{RT p:=@ LT}} fn f {label entry:{call g \
     @:=p}} fn main {label entry:{x:=@ call f}}"
    "-|a|b" "-|-|b";
  equivalent
    "alphabet:(a,b) fn main {label first:{@:=const a break @:=const b} label \
     second:{RT @:=const b}}"
    "|-|" "a|b|";
  equivalent
    "alphabet:(a,b) fn main {label loop:{@:=const a RT continue if @==const b} \
     label done:{@:=const a}}"
    "|-|b,b,-" "a,a,a|a|";
  equivalent
    "alphabet:(a,b) fn f {label entry:{jump done if flag==const a flag:=const \
     a @:=const a return} label done:{@:=const b}} fn main {label loop:{call f \
     RT continue if @==const b}}"
    "|-|b,b,-" "a,a,a|-|";
  equivalent
    "alphabet:(a,b) fn f {label entry:{RT}} fn main {label loop:{call f \
     continue if @==const b} label done:{@:=const a}}"
    "|-|b,b,-" "-,b,b|a|";
  equivalent
    "alphabet:(a,b) fn f {label entry:{RT}} fn main {label loop:{call f break \
     @:=const b} label done:{@:=const a}}"
    "|-|" "-|a|";
  equivalent
    "alphabet:(a,b) fn f {label entry:{__flat0_v:=const b @:=__flat0_v}} fn \
     main {label __init0:{__flat0___flat0_v:=const a call f \
     @:=__flat0___flat0_v}}"
    "|-|" "|a|";
  equivalent
    "alphabet:(m,a,b,x) fn main {label entry:{mark:=@ jump scan} label \
     scan:{RT cur:=@ jump done if cur==const x jump scan} label \
     done:{@:=mark}}"
    "-|m|a,b,x,-" "-,m,a,b|m|-";
  equivalent
    "alphabet:(a,b,c) fn main {label entry:{jump scan_b} label scan_b:{RT v:=@ \
     jump scan_c if v==const b jump scan_b} label scan_c:{RT v:=@ jump done if \
     v==const c jump scan_b} label done:{}}"
    "-|a|b,c" "-,a,b|c|";
  equivalent
    "alphabet:(a) fn some_function {label entry_label:{v_1:=const a @:=v_1}} \
     fn main {label start:{call some_function()}}"
    "|-|" "|a|";
  let invalid =
    "alphabet:(a) fn f {label entry:{call f}} fn main {label entry:{call f}}"
  in
  check "recursion rejected"
    (Result.is_error (I.make (get (I.parse_code invalid)) (tape "|-|")));
  List.iter
    (fun code ->
      check "IR validation"
        (Result.is_error (I.make (get (I.parse_code code)) (tape "|-|"))))
    [
      "alphabet:() fn main {}";
      "alphabet:(a) fn main {label entry:{jump missing}}";
      "alphabet:(a) fn main {label entry:{@:=const b}}";
      "alphabet:(a) fn main {label entry:{} label entry:{}}";
    ];
  let p =
    get
      (I.parse_code
         "alphabet:(a) fn f {label entry:{RT}} fn main {label entry:{call f \
          @:=const a}}")
  in
  let s = get (I.make p (tape "|-|")) in
  let s =
    match get (I.step s ()) with
    | Utils.Machine.Continue { next; _ } -> next
    | _ -> failwith "call step"
  in
  let restored = get (I.restore (I.snapshot s)) in
  check "IR snapshot stack" (T.tape_equal (run I.step s) (run I.step restored));
  let p = get (J.parse_code "alphabet:(a,b) v:=@ RT jump if v==@ 1 @:=v") in
  let s = get (J.make p (tape "|a|b")) in
  let restored = get (J.restore (J.snapshot s)) in
  check "jump snapshot" (T.tape_equal (run J.step s) (run J.step restored));
  check "out of range jump"
    (Result.is_error
       (J.make (get (J.parse_code "alphabet:(a) jump 2")) (tape "|-|")));
  let p = get (J.parse_code "alphabet:(a,b) @:=missing") in
  check "blank env initialization"
    (T.tape_equal
       (run_jump p (tape "|a|"))
       (run_tm (get (Compile_tm.compile p)) (tape "|a|")));
  List.iter
    (fun ns ->
      check "number tape roundtrip"
        (C.read_right_one_usize (get (C.write_usize ns)) = Some ns))
    [ []; [ 0 ]; [ 1 ]; [ 2 ]; [ 1; 1 ]; [ 1; 2; 3 ] ];
  helper
    (fun _ -> C.move_right_till_x_n_times 3)
    "|-|l,x,-,x,x,-,-,x,l,x,l,-" "-,l,x,-,x|x|-,-,x,l,x,l,-";
  helper
    (fun _ -> C.move_left_till_x_n_times 2)
    "-,l,x,-,x,x,-,-,x,l|x|l,-" "-,l,x,-,x|x|-,-,x,l,x,l,-";
  List.iter
    (fun (a, b) -> helper (fun ctx -> C.shift_left_x ctx C.b) a b)
    [
      ("l,x,-,-,-|x|", "l|-|-,-,x,-");
      ("l,x|x|", "l|x|-");
      ("l,x,x|x|", "l,x|x|-");
      ("l,x,l,x,-,-,-|x|", "l,x,l|-|-,-,x,-");
    ];
  helper (fun ctx -> C.shift_left_x ctx C.x) "l,x,-,-,-|x|" "l|-|-,-,x,x";
  helper (fun ctx -> C.shift_left_x ctx C.b) "l,x,-,l,-|l|" "l|-|l,-,l,-";
  helper
    (fun ctx -> C.shift_left_x_n_times ctx 2)
    "l|x|-,l,-,x,l,x" "l|-|l,-,x,l,x,-";
  helper (fun ctx -> C.shift_left_x_n_times ctx 2) "l|x|x,x" "l|x|x,-";
  List.iter
    (fun (a, b) -> helper C.swap_tuple a b)
    [
      ("|x|-,l,l,x,-,l,x", "|x|-,l,x,-,l,l,x");
      ("|x|-,l,x,-,l,l,x", "|x|-,l,l,x,-,l,x");
      ("|x|x,x", "|x|x,x");
      ("|x|x,-,x", "|x|-,x,x");
      ("|x|-,x,x", "|x|x,-,x");
    ];
  helper (fun ctx -> C.rotate ctx 3) "|x|-,x,x,l,x" "|x|x,l,x,-,x";
  helper
    (fun ctx -> C.copy_to_end ctx 1)
    "|x|-,l,l,x,l,-,l,x" "|x|-,l,l,x,l,-,l,x,-,l,l,x";
  helper (fun ctx -> C.copy_to_end ctx 0) "|x|x" "|x|x,x";
  helper (fun ctx -> C.copy_n_times ctx 0) "|x|-,l,l,x" "|x|-,l,l,x";
  helper
    (fun ctx -> C.copy_n_times ctx 2)
    "|x|-,l,l,x" "|x|-,l,l,x,-,l,l,x,-,l,l,x";
  List.iter
    (fun ns ->
      let ctx = C.create () in
      let p = C.wrap_function ctx (C.pred_tuple ctx) in
      let expected =
        match ns with
        | 0 :: tail -> tail
        | n :: tail -> (n - 1) :: tail
        | [] -> []
      in
      check "pred tuple"
        (T.tape_equal
           (run_ir p (get (C.write_usize ns)))
           (get (C.write_usize expected))))
    [ [ 1; 1 ]; [ 1; 0 ]; [ 1 ]; [ 0; 2 ]; [ 0 ] ];
  recursive R.Zero [] 0;
  recursive R.Succ [ 2 ] 3;
  recursive (R.Proj (3, 0)) [ 2; 0; 3 ] 2;
  recursive (R.Proj (3, 1)) [ 4; 0; 1 ] 0;
  recursive (R.Proj (3, 2)) [ 4; 0; 2 ] 2;
  recursive (R.Proj (1, 0)) [ 0 ] 0;
  recursive (R.Comp (R.Succ, [ R.Succ ])) [ 2 ] 4;
  recursive (R.Comp (R.Succ, [ R.Zero ])) [] 1;
  recursive (R.Comp (R.Proj (2, 0), [ R.Zero; R.Zero ])) [] 0;
  recursive (R.Comp (R.Proj (2, 0), [ R.Succ; R.Proj (1, 0) ])) [ 2 ] 3;
  recursive (R.Comp (R.Proj (2, 1), [ R.Succ; R.Proj (1, 0) ])) [ 2 ] 2;
  let recursive_ir f args expected =
    let p = get (C.compile f) in
    let t = get (C.write_usize args) in
    let out = run_ir p t in
    check
      ("primitive/mu result " ^ R.print f ^ " tape=" ^ T.print_tape out)
      (C.read_right_one_usize out = Some [ expected ]);
    let jump = get (Compile_jump.compile p) in
    check "primitive/mu jump equivalence" (T.tape_equal out (run_jump jump t));
    let tm = get (Compile_tm.compile jump) in
    check "primitive/mu Turing equivalence" (T.tape_equal out (run_tm tm t));
    check "reachable compiler state count" (List.length tm.T.code < 100000)
  in
  let identity = R.Prim (R.Zero, R.Comp (R.Succ, [ R.Proj (2, 0) ])) in
  recursive_ir identity [ 0 ] 0;
  recursive_ir identity [ 1 ] 1;
  recursive_ir identity [ 3 ] 3;
  recursive_ir (R.Mu (R.Proj (2, 0))) [ 2 ] 0;
  let pred = R.Prim (R.Zero, R.Proj (2, 1)) in
  let subtraction = R.Prim (R.Proj (1, 0), R.Comp (pred, [ R.Proj (3, 0) ])) in
  recursive_ir (R.Mu subtraction) [ 2 ] 2;
  recursive_ir
    (R.Prim (R.Proj (1, 0), R.Comp (R.Succ, [ R.Proj (3, 0) ])))
    [ 3; 2 ] 5;
  recursive_ir (R.Comp (subtraction, [ R.Succ; R.Proj (1, 0) ])) [ 2 ] 0;
  recursive_ir (R.Comp (subtraction, [ R.Proj (1, 0); R.Succ ])) [ 2 ] 1;
  List.iter
    (fun n ->
      List.iter
        (fun m -> recursive_ir subtraction [ n; m ] (max 0 (m - n)))
        [ 0; 1; 3 ])
    [ 0; 1; 3 ];
  let divergent = R.Mu (R.Comp (R.Succ, [ R.Proj (1, 0) ])) in
  let ir = get (C.compile divergent) in
  let tm = get (Rec_to_tm.compile divergent) in
  let t = get (C.write_usize []) in
  let still_running step state =
    let rec loop remaining state =
      if remaining = 0 then true
      else
        match get (step state ()) with
        | Utils.Machine.Continue { next; _ } -> loop (remaining - 1) next
        | Utils.Machine.Halt _ -> false
    in
    loop 10000 state
  in
  check "mu divergence in IR" (still_running I.step (get (I.make ir t)));
  check "mu divergence in Turing" (still_running T.step (get (T.make tm t)));
  let encoded = get (C.api.encode_ainput "(2,0)") in
  check "compiler encoded input" (encoded = "|x|-,l,l,-,x");
  let compiled = get (Rec_to_tm.api.compile_code "SUCC") in
  let output =
    run_tm
      (get (T.parse_code compiled))
      (tape (get (Rec_to_tm.api.encode_ainput "(2)")))
  in
  check "full chain text API"
    (get (Rec_to_tm.api.decode_foutput (T.print_tape output)) = "3");
  check "output decode wrong head"
    (Result.is_error (C.decode_foutput (tape "|-|x")));
  check "output decode tuple"
    (Result.is_error (C.decode_foutput (get (C.write_usize [ 1; 2 ]))));
  print_endline "IR, helpers, compiler roundtrips and equivalence tests passed"
