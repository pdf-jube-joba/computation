open Logic_circuit
module Machine_contract : Utils.Machine.S = Logic_circuit.S

let ok = function Ok x -> x | Error e -> failwith e
let bad = function Error _ -> () | Ok _ -> failwith "Expected error"
let pin n p = (n, p)
let advance c xs = ok (step c xs)

let value c =
  match get_otputs c with
  | [ (_, b) ] -> b
  | _ -> failwith "Expected one output"

let rec clocks n c xs = if n = 0 then c else clocks (n - 1) (advance c xs) xs

let () =
  let g = new_gate And false in
  assert (get_inpins g = [ pin "AND" "IN0"; pin "AND" "IN1" ]);
  assert (get_otpins g = [ pin "AND" "OUT" ]);
  assert (value (advance g [ (pin "AND" "IN0", true); (pin "AND" "IN1", true) ]));
  assert (not (value (advance g [ (pin "AND" "IN0", true) ])));
  List.iter
    (fun a ->
      List.iter
        (fun b ->
          assert (
            value
              (advance (new_gate And false)
                 [ (pin "x" "IN0", a); (pin "y" "IN1", b) ])
            = (a && b));
          assert (
            value
              (advance (new_gate Or true)
                 [ (pin "x" "IN0", a); (pin "y" "IN1", b) ])
            = (a || b)))
        [ false; true ])
    [ false; true ];
  assert (value (advance (new_gate Cst true) []));
  assert (value (advance (new_gate Not false) []));
  assert (
    get_otputs (advance (new_gate Br false) [ (pin "x" "IN", true) ])
    = [ (pin "BR" "OUT0", true); (pin "BR" "OUT1", true) ]);
  assert (value (advance (new_gate Delay false) [ (pin "x" "IN", true) ]));
  let end_gate = advance (new_gate End false) [ (pin "x" "IN", true) ] in
  assert (get_otputs end_gate = [] && state_as_gate end_gate = Some true);
  (* First matching signal wins; gate input names are intentionally ignored. *)
  assert (
    value
      (advance (new_gate Delay false)
         [ (pin "x" "IN", true); (pin "y" "IN", false) ]));
  let mix =
    ok
      (new_mix "MIX"
         [ ("A", new_gate And false); ("B", new_gate Or false) ]
         [ (pin "A" "OUT", pin "B" "IN0") ]
         [] [])
  in
  assert (get_inpins mix = [ pin "A" "IN0"; pin "A" "IN1"; pin "B" "IN1" ]);
  assert (get_otpins mix = [ pin "B" "OUT" ]);
  let first =
    advance mix
      [ (pin "A" "IN0", true); (pin "A" "IN1", true); (pin "B" "IN0", true) ]
  in
  assert (not (value first));
  assert (value (advance first []));
  let code =
    "/* comment */ graph: main {in {I} out {O=A.OUT} A, NOT_F {IN=I}}"
  in
  let main = ok (parse_code code) in
  assert (
    ok (parse_code ("/* nested opener /* closes at first terminator */" ^ code))
    = main);
  bad (parse_code ("/* nested opener /* */ */" ^ code));
  bad (parse_code "/* unclosed graph: main {in {} out {}}");
  assert (ok (parse_code ("// /* ignored opener\n" ^ code)) = main);
  assert (print_code main = "logic_circuit");
  assert (get_inpins main = [ pin "main" "I" ]);
  assert (print_signal (get_otputs (advance main [])) = "main.O=#true");
  assert (not (value (advance main [ (pin "main" "I", true) ])));
  assert (not (value (advance main [ (pin "A" "IN", true) ])));
  assert (
    value
      (advance main
         [ (pin "main" "UNKNOWN", true); (pin "UNKNOWN" "IN", true) ]));
  (* Maps targeting absent vertices are accepted, exactly as in Rust. *)
  let permissive =
    ok (parse_code "graph: main { in {A, d} out {a=b.c} A, AND_T {} }")
  in
  assert (get_otpins permissive = [ pin "A" "OUT" ]);
  let permissive_json = render permissive |> Yojson.Safe.to_string in
  assert (String.length permissive_json > 0);
  bad (parse_code "graph: main {in {} out {} A, NOT_F {} A, NOT_T {}}");
  bad (parse_code "graph: main {in {} out {} A, NOT_F {IN=absent.OUT}}");
  bad (parse_code "graph: main {in {} out {} A, NOT_F {IN=X}}");
  bad (parse_code "graph: main {in {} out {} A, NO_SUCH_GATE {}}");
  bad
    (parse_code
       "graph: main {in {} out {} A, BR_F {} B, DLY_F {IN=A.OUT0} C, DLY_F \
        {IN=A.OUT0}}");
  bad
    (parse_code
       "graph: main {in {} out {} A, DLY_F {} B, AND_F {IN0=A.OUT IN0=A.OUT}}");
  bad (parse_code "graph: _bad { in {} out {} } trailing");
  bad (parse_code "graph: 123 {in {} out {}}");
  bad (parse_code "");
  let duplicate = ok (parse_code (code ^ " graph: main {in {} out {}}")) in
  assert (duplicate = main);
  let signal = ok (parse_rinput " A.IN=#true, B.OUT=#false ") in
  assert (ok (parse_rinput "\tA.IN=\r#true\n, B.OUT=#false\012") = signal);
  assert (print_signal signal = "A.IN=#true, B.OUT=#false");
  assert (ok (parse_rinput "  ") = []);
  let whitespace = [ "\t"; "\n"; "\012"; "\r"; " " ] in
  List.iter
    (fun ws ->
      assert (ok (parse_rinput ws) = []);
      assert (ok (parse_ainput ws) = ());
      assert (
        ok (parse_rinput (ws ^ "A.IN=" ^ ws ^ "#true" ^ ws))
        = [ (pin "A" "IN", true) ]))
    whitespace;
  bad (parse_rinput "A.\194\160IN=#true");
  bad (parse_rinput "\227\128\128A.IN=#true");
  bad (parse_rinput "\226\128\139A.IN=#true");
  bad (parse_ainput "\239\187\191");
  List.iter
    (fun s -> bad (parse_rinput s))
    [ "A.IN=true"; "A=#true"; "A.IN=#true,"; "0.IN=#true"; "A .IN=#true" ];
  bad (parse_ainput "unexpected");
  let saved = snapshot first in
  let decoded =
    Yojson.Safe.from_string (Yojson.Safe.to_string saved) |> restore |> ok
  in
  assert (decoded = first && advance decoded [] = advance first []);
  (* Rust's public Mix struct and snapshot decoder accept duplicate vertex
     names even though new_mix rejects them. Its name map steps only the first. *)
  let duplicate_state =
    Mix
      {
        kind = "DUP";
        verts = [ ("A", new_gate Not false); ("A", new_gate Delay true) ];
        edges = [];
        inpin_maps = [];
        otpin_maps = [];
      }
  in
  let duplicate_state = ok (restore (snapshot duplicate_state)) in
  assert (
    get_otputs (advance duplicate_state [])
    = [ (pin "A" "OUT", true); (pin "A" "OUT", true) ]);
  bad (restore (`String "invalid"));
  let invalid_identifiers = ok (new_mix "123" [] [] [] []) in
  bad (restore (snapshot invalid_identifiers));
  (match ok (S.step main []) with
  | Utils.Machine.Continue { next; output } ->
      assert (output = get_otputs next);
      assert (S.routput_json output = `String "main.O=#true")
  | _ -> failwith "Circuits never halt");
  (match render first with
  | `List [ _; _; _; _ ] -> ()
  | _ -> failwith "Expected four render blocks");
  let library = ok (Example.utils_map ()) in
  assert (
    List.map fst (Manipulation.init_maps ())
    = [
        "NOT_T";
        "NOT_F";
        "AND_T";
        "AND_F";
        "OR_T";
        "OR_F";
        "CST_T";
        "CST_F";
        "BR_T";
        "BR_F";
        "END";
        "DLY_T";
        "DLY_F";
      ]);
  assert (Manipulation.by_const "my_gate_1" = "my_gate_1");
  (try
     ignore (Manipulation.by_const "123");
     failwith "Expected invalid identifier"
   with Invalid_argument _ -> ());
  assert (List.length library = 19);
  let one_shot = List.assoc "one_shot" library in
  let pulse = advance one_shot [ (pin "one_shot" "I", true) ] in
  assert (not (value pulse));
  assert (value (advance pulse [ (pin "one_shot" "I", true) ]));
  assert (not (value (clocks 6 one_shot [ (pin "one_shot" "I", true) ])));
  let xor = List.assoc "XOR" library in
  List.iter
    (fun a ->
      List.iter
        (fun b ->
          assert (
            value (clocks 6 xor [ (pin "XOR" "IN0", a); (pin "XOR" "IN1", b) ])
            = (a <> b)))
        [ false; true ])
    [ false; true ];
  let adder = List.assoc "ADDR" library in
  (* Rust sends instance X to the nested circuit whose kind is XOR. Its input
     resolver leaves X unchanged and finds no vertex X, so the sum stays false. *)
  List.iter
    (fun a ->
      List.iter
        (fun b ->
          assert (
            get_otputs
              (clocks 12 adder [ (pin "ADDR" "I0", a); (pin "ADDR" "I1", b) ])
            = [ (pin "ADDR" "S", false); (pin "ADDR" "C", a && b) ]))
        [ false; true ])
    [ false; true ];
  List.iter
    (fun name ->
      let c = clocks 20 (List.assoc name library) [] in
      assert (ok (restore (snapshot c)) = c);
      assert (List.length (as_graph_group c).verts > 0))
    [ "RS_latch"; "DFF" ];
  let iter = ok (parse_code "iter: main {DLY_F, next {OUT=IN} prev {}}") in
  assert (get_inpins iter = [] && get_otputs iter = [] && advance iter [] = iter);
  (* Typed callers can seed the decimal index that Rust's text Identifier
     cannot express. Generated _n_ labels remain distinct and ignored. *)
  let iter = advance iter [ (pin "0" "IN", true) ] in
  (match iter with
  | Iter i -> assert (List.length i.used = 1)
  | _ -> assert false);
  assert (get_inpins iter = [ pin "_0_" "IN" ]);
  let extended = advance iter [] in
  (match extended with
  | Iter i -> assert (List.length i.used = 2)
  | _ -> assert false);
  assert (
    (as_graph_group extended).edges = [ (pin "_0_" "OUT", pin "_1_" "IN") ]);
  assert (ok (restore (snapshot extended)) = extended);
  bad (step iter [ (pin "10" "IN", true) ]);
  let backwards =
    ok (new_iter "back" (new_gate Delay false) [] [ ("OUT", "IN") ])
  in
  let backwards = advance backwards [ (pin "0" "IN", true) ] in
  bad (step backwards []);
  print_endline "logic_circuit tests passed"
