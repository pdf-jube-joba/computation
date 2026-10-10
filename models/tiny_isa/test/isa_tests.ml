module I = Tiny_isa
module A = Tiny_isa.Symbolic_asm
module C = Tiny_isa.Cfg_vreg
module N = Utils.Number
module Op = Tiny_isa.Isa_common

let ok = function Ok v -> v | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "Expected error"

let next_i s =
  match ok (I.step s ()) with
  | Utils.Machine.Continue { next; output = () } -> next
  | _ -> failwith "Expected continue"

let next_a s =
  match ok (A.step s ()) with
  | Utils.Machine.Continue { next; output = () } -> next
  | _ -> failwith "Expected continue"

let next_c s =
  match ok (C.step s ()) with
  | Utils.Machine.Continue { next; output = () } -> next
  | _ -> failwith "Expected continue"

let rec run_i fuel s =
  if fuel = 0 then failwith "ISA did not halt"
  else
    match ok (I.step s ()) with
    | Utils.Machine.Halt output -> output
    | Continue { next; _ } -> run_i (fuel - 1) next

let rec run_a fuel s =
  if fuel = 0 then failwith "ASM did not halt"
  else
    match ok (A.step s ()) with
    | Utils.Machine.Halt output -> output
    | Continue { next; _ } -> run_a (fuel - 1) next

let image : I.ainput = { register = Array.make 8 N.zero; memory = [] }
let asm_input : A.ainput = { regs = Array.make 8 N.zero; data_extension = [] }
let machine ops = ok (I.make (List.map I.encode ops) image)
let assembly source = ok (A.make (ok (A.parse_code source)) asm_input)

let () =
  let (_ : (module Utils.Machine.S)) = (module I.Machine) in
  let (_ : (module Utils.Machine.S)) = (module A.Machine) in
  let (_ : (module Utils.Machine.S)) = (module C.Machine) in
  let roundtrip inst = assert (ok (I.decode (I.encode inst)) = inst) in
  List.iter roundtrip [ Op.Nop; Halt; Rst ];
  for d = 0 to 7 do
    for s = 0 to 7 do
      List.iter roundtrip
        [ Op.Ld (d, s); St (d, s); Eq (d, s); Lt (d, s); Gt (d, s) ];
      List.iter
        (fun f ->
          List.iter roundtrip [ Op.Mov (f, d, s); Add (f, d, s); Sub (f, d, s) ])
        [ false; true ]
    done;
    List.iter
      (fun f ->
        List.iter
          (fun n ->
            List.iter roundtrip
              [ Op.Ldi (f, d, n); Addi (f, d, n); Subi (f, d, n) ])
          [ N.zero; N.one; "65536"; "340282366920938463463374607431768211456" ])
      [ false; true ]
  done;
  assert (ok (I.decode "64") = Op.Halt);
  assert (ok (I.decode "63") = Op.Rst);
  List.iter
    (fun n -> error (I.decode (N.of_int n)))
    [ 1; 65; 320; 513 + 64; (17 * 256) + 64; (8 * 256) + 128 + 1 ];
  assert (
    ok (I.decode (N.add (N.mul "99999999999999999999999999" "65536") "64"))
    = Op.Halt);
  let output =
    run_i 60
      (machine
         [
           Op.Ldi (false, 1, "3");
           Ldi (false, 2, "4");
           Add (false, 1, 2);
           Sub (false, 2, 1);
           Eq (2, 3);
           Ldi (true, 4, "17");
           Lt (2, 1);
           Addi (true, 4, "2");
           Gt (1, 4);
           Subi (true, 4, "5");
           Rst;
           Ldi (true, 5, "100");
           Mov (false, 6, 4);
           Halt;
         ])
  in
  assert (output.register.(1) = "7");
  assert (output.register.(2) = "0");
  assert (output.register.(4) = "19");
  assert (output.register.(5) = "0");
  assert (output.register.(6) = "19");
  assert (output.register.(0) = "13");
  let output =
    run_i 15 (machine [ Op.Ldi (false, 0, "2"); Ldi (false, 1, "999"); Halt ])
  in
  assert (output.register.(1) = "0" && output.register.(0) = "2");
  (* Store into the next instruction: the machine must decode the new halt word. *)
  let regs = Array.make 8 N.zero in
  regs.(1) <- "64";
  regs.(2) <- "1";
  let output =
    run_i 4
      (ok
         (I.make
            [ List.hd (List.map I.encode [ Op.St (1, 2) ]); N.zero ]
            { register = regs; memory = [] }))
  in
  assert (output.register.(0) = "1");
  let code = List.map I.encode [ Op.Ld (1, 2); St (1, 3); Halt ] in
  let regs = Array.make 8 N.zero in
  regs.(2) <- "3";
  regs.(3) <- "6";
  let output =
    run_i 10 (ok (I.make code { register = regs; memory = [ "42" ] }))
  in
  assert (output.memory = [ "42"; "0"; "0"; "42" ]);
  let s = machine [ Op.Nop ] in
  let s = next_i s in
  let s = next_i s in
  assert (s.regs.(0) = "2");
  let s =
    machine [ Op.Ldi (false, 1, "123456789012345678901234567890"); Halt ]
  in
  let snapshot = I.snapshot s in
  let restored =
    ok (I.restore (Yojson.Safe.from_string (Yojson.Safe.to_string snapshot)))
  in
  assert (I.snapshot (next_i s) = I.snapshot (next_i restored));
  assert (I.snapshot s = snapshot);
  let input = ok (I.parse_ainput "\n0, 1,2,3,4,5,6,7,\n\n8\n9") in
  assert (input.memory = [ "8"; "9" ]);
  assert (
    ok
      (I.parse_ainput
         (match I.foutput_json input with `String s -> s | _ -> assert false))
    = input);
  assert (
    ok (I.parse_code (I.print [ "0"; "64"; "123456789012345678901234567890" ]))
    = [ "0"; "64"; "123456789012345678901234567890" ]);
  error (I.parse_ainput "0,1");
  error (I.parse_ainput "x,0,0,0,0,0,0,0");
  error (I.parse_code "64 x");
  let text =
    ".equ N #7\n\
     .text\n\
     @main:\n\
     ldi %r1 N;\n\
     add %r1 #2;\n\
     ldi %r2 @cell;\n\
     st %r1 %r2;\n\
     ld %r3 %r2;\n\
     eq %r1 %r3;\n\
     jump.f @end;\n\
     ldi %r4 #99;\n\
     @end:\n\
     halt;\n\
     .data\n\
     @cell:\n\
     0\n\
     @THIS\n\
     @CODE_LEN\n\
     @DATA_LEN\n\
     @main"
  in
  let code = ok (A.parse_code text) in
  assert (ok (A.parse_code (A.print code)) = code);
  let output = run_a 30 (ok (A.make code asm_input)) in
  assert (
    output.regs.(1) = "9"
    && output.regs.(3) = "9"
    && output.regs.(4) = "0"
    && output.regs.(0) = "8");
  assert (output.data_memory = [ "9"; "1"; "9"; "5"; "0" ]);
  let input = ok (A.parse_ainput "0, 0, 0, 0, 0, 0, 0, 0\n100") in
  let output = run_a 20 (ok (A.make code input)) in
  assert (List.nth output.data_memory 5 = "100");
  assert (
    ok
      (A.parse_output
         (match A.foutput_json output with `String s -> s | _ -> assert false))
    = output);
  let output =
    run_a 30
      (assembly
         ".data\n\
          @a:\n\
          1\n\
          .text\n\
          ldi %1 @a\n\
          .data\n\
          @b:\n\
          2\n\
          .text\n\
          ldi %2 @b;\n\
          ld %3 %2;\n\
          add %3 #5;\n\
          sub %3 #100;\n\
          mov %4 %3;\n\
          halt;")
  in
  assert (
    output.regs.(1) = "0"
    && output.regs.(2) = "1"
    && output.regs.(3) = "0"
    && output.data_memory = [ "1"; "2" ]);
  let output =
    run_a 15
      (assembly "ldi.f %r1 @unknown;\nldi %r2 #4;\njump %r2;\nldi %r1 #99;\n")
  in
  assert (output.regs.(0) = "4" && output.regs.(1) = "0");
  let output =
    run_a 20
      (assembly
         "ldi %r1 #1;\n\
          @loop:\n\
          add %r1 #1;\n\
          ldi %r2 #4;\n\
          lt %r1 %r2;\n\
          jump.f @loop;\n\
          halt;")
  in
  assert (output.regs.(1) = "4");
  let s = assembly "nop;\nhalt;" in
  let snap = A.snapshot s in
  let restored =
    ok (A.restore (Yojson.Safe.from_string (Yojson.Safe.to_string snap)))
  in
  assert (A.snapshot (next_a s) = A.snapshot (next_a restored));
  assert (A.snapshot s = snap);
  List.iter
    (fun text -> error (A.make (ok (A.parse_code text)) asm_input))
    [
      "@THIS:\nhalt;";
      ".equ CODE_LEN #1";
      "@x:\n@x:\nhalt;";
      ".equ n #1\n@n:\nhalt;";
      "nop.f;";
      "ld.f %r1 %r2;";
      "eq.f %r1 %r2;";
      "mov %r1 #2;";
      "jump;";
      "unknown;";
      ".data\n@unknown";
    ];
  List.iter
    (fun text -> error (A.parse_code text))
    [ ".bss"; "ldi %r8 #1;"; ".data\n1 2"; "@:" ];
  error (A.step (assembly "ldi %r1 missing;") ());
  let program = "@cell 3\n@main {\n %v0 := *@cell_invalid;\n goto @main;\n}" in
  error (C.parse_code program);
  let program =
    "@cell 3\n\
     @main {\n\
    \ %v0 := *[@cell];\n\
    \ %v2 := %v0 + #4;\n\
    \ [@cell] := %v2;\n\
    \ if %v2 = #7 then @done;\n\
    \ goto @main;\n\
     }\n\
     @done {\n\
    \ %v1 := #1;\n\
    \ %v1 := %v1 - #9;\n\
    \ [#3] := %v1;\n\
    \ goto @main;\n\
     }"
  in
  let code = ok (C.parse_code program) in
  assert (ok (C.parse_code (C.print code)) = code);
  let s = ok (C.make code (ok (C.parse_ainput "(8, 9,)"))) in
  let s1 = next_c s in
  assert (
    s1.current_block = 1
    && s1.vregs = [| "3"; "0"; "7" |]
    && s1.memory = [| "7"; "8"; "9" |]);
  assert (s.current_block = 0 && s.memory = [| "3"; "8"; "9" |]);
  let s2 = next_c s1 in
  assert (
    s2.current_block = 0
    && s2.memory = [| "7"; "8"; "9"; "0" |]
    && s2.vregs.(1) = "0");
  let restored =
    ok
      (C.restore
         (Yojson.Safe.from_string (Yojson.Safe.to_string (C.snapshot s1))))
  in
  assert (C.snapshot (next_c s1) = C.snapshot (next_c restored));
  let program =
    "@main {\n\
    \ %v0 := #2;\n\
    \ if #2 > #1 then @first;\n\
    \ if #1 < #2 then @second;\n\
    \ goto @main;\n\
     }\n\
     @first {\n\
    \ goto @main;\n\
     }\n\
     @second {\n\
    \ goto @main;\n\
     }"
  in
  let s = next_c (ok (C.make (ok (C.parse_code program)) [])) in
  assert (s.current_block = 1);
  let program =
    "@main {\n\
    \ %v0 := @next;\n\
    \ goto %v0;\n\
     }\n\
     @next {\n\
    \ [#4] := *[#99];\n\
    \ goto #0;\n\
     }"
  in
  let s = next_c (ok (C.make (ok (C.parse_code program)) [])) in
  assert (s.current_block = 1 && s.vregs.(0) = "1");
  let s = next_c s in
  assert (s.memory = [| "0"; "0"; "0"; "0"; "0" |]);
  let code =
    ok
      (C.parse_code
         "@shared 4\n\
          @main {\n\
         \ %v0 := @shared;\n\
         \ goto @main;\n\
          }\n\
          @shared {\n\
         \ goto @main;\n\
          }")
  in
  assert ((next_c (ok (C.make code []))).vregs.(0) = "0");
  List.iter
    (fun text -> error (C.make (ok (C.parse_code text)) []))
    [
      "";
      "@main {\n goto @main;\n}\n@main {\n goto @main;\n}";
      "@x 0\n@x 1\n@main {\n goto @main;\n}";
    ];
  List.iter
    (fun text -> error (C.parse_code text))
    [
      "@main {\n}";
      "@main {\n goto #0;\n goto #0;\n}";
      "@main {\n goto #0;\n %v0 := #1;\n}";
      "@main {\n goto #0;";
      "@main {\n %v0 := #1 * #2;\n goto #0;\n}";
    ];
  List.iter
    (fun target ->
      let s =
        ok (C.make (ok (C.parse_code ("@main {\n goto " ^ target ^ ";\n}"))) [])
      in
      error (C.step s ()))
    [ "#9"; "@missing" ];
  let s = { s1 with halted = true } in
  assert (
    match ok (C.step s ()) with
    | Utils.Machine.Halt ns -> ns = [ "7"; "8"; "9" ]
    | _ -> false);
  assert (C.foutput_json [ "1"; "2" ] = `String "(1, 2)");
  assert (ok (C.parse_ainput "(,1,,2,)") = [ "1"; "2" ]);
  List.iter (fun text -> error (C.parse_ainput text)) [ "1,2"; "(1;x)"; "(-1)" ];
  error (I.parse_rinput "x");
  error (A.parse_rinput "x");
  error (C.parse_rinput "x");
  assert (
    match I.render (machine [ Op.Halt ]) with
    | `List [ _; _; _ ] -> true
    | _ -> false);
  assert (
    match A.render (assembly "halt;") with
    | `List [ _; _; _ ] -> true
    | _ -> false);
  assert (match C.render s1 with `List [ _; _; _; _ ] -> true | _ -> false);
  assert (ok (I.parse_code "+64") = [ "64" ]);
  assert ((run_a 5 (assembly "ldi %r1 #+7;\nhalt;")).regs.(1) = "7");
  assert (ok (C.parse_ainput "(+1,+2)") = [ "1"; "2" ]);
  print_endline "tiny ISA, symbolic assembly, and CFG tests passed"
