open Logic_circuit.Main
open Logic_circuit.Circuit
module Machine_contract : Utils.Machine.S = Logic_circuit.Main

let ok = function Ok x -> x | Error e -> failwith e
let bad = function Error _ -> () | Ok _ -> failwith "Expected error"
let pin n p = (Internal n, p)
let ext n p = (External n, p)
let wire n p = (n, p)
let advance c xs = ok (step c xs)

let value c =
  match get_otputs c with
  | [ (_, b) ] -> b
  | _ -> failwith "Expected one output"

let rec clocks n c xs = if n = 0 then c else clocks (n - 1) (advance c xs) xs

let () =
  let g = new_gate And false in
  assert (get_inpins g = [ ext "AND" "IN0"; ext "AND" "IN1" ]);
  assert (get_otpins g = [ ext "AND" "OUT" ]);
  assert (value (advance g [ (ext "AND" "IN0", true); (ext "AND" "IN1", true) ]));
  assert (not (value (advance g [ (ext "AND" "IN0", true) ])));
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
    = [ (ext "BR" "OUT0", true); (ext "BR" "OUT1", true) ]);
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
         [ (wire "A" "OUT", wire "B" "IN0") ]
         [] [])
  in
  assert (get_inpins mix = [ pin "A" "IN0"; pin "A" "IN1"; pin "B" "IN1" ]);
  assert (get_otpins mix = [ pin "B" "OUT" ]);
  let collision =
    ok (new_mix "same" [ ("same", new_gate Delay false) ] [] [] [])
  in
  assert (value (advance collision [ (pin "same" "IN", true) ]));
  assert (not (value (advance collision [ (ext "same" "IN", true) ])));
  let first =
    advance mix
      [ (pin "A" "IN0", true); (pin "A" "IN1", true); (pin "B" "IN0", true) ]
  in
  assert (not (value first));
  assert (value (advance first []));
  let code =
    "/* comment */ graph: main {in {I} out {O=A.OUT} A, NOT_F {IN=I}}"
  in
  let main = ok (Code.parse code) in
  assert (get_inpins main = [ ext "main" "I" ]);
  assert (Signal.print (get_otputs (advance main [])) = "@main.O=#true");
  assert (not (value (advance main [ (ext "main" "I", true) ])));
  assert (not (value (advance main [ (pin "A" "IN", true) ])));
  assert (
    value
      (advance main
         [ (ext "main" "UNKNOWN", true); (pin "UNKNOWN" "IN", true) ]));
  let saved = snapshot first in
  let decoded =
    Yojson.Safe.from_string (Yojson.Safe.to_string saved) |> restore |> ok
  in
  assert (decoded = first && advance decoded [] = advance first []);
  (* A restored topology must have unambiguous vertex identities. *)
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
  bad (restore (snapshot duplicate_state));
  bad (restore (`String "invalid"));
  bad (new_mix "123" [] [] [] []);
  (match ok (Logic_circuit.Main.step main []) with
  | Utils.Machine.Continue { next; output } ->
      assert (output = get_otputs next);
      assert (Logic_circuit.Main.Routput.print output = "@main.O=#true")
  | _ -> failwith "Circuits never halt");
  let library = ok (Logic_circuit.Example.utils_map ()) in
  assert (
    List.map fst (Logic_circuit.Parser.init_maps ())
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
  assert (Logic_circuit.Parser.by_const "my_gate_1" = "my_gate_1");
  (try
     ignore (Logic_circuit.Parser.by_const "123");
     failwith "Expected invalid identifier"
   with Invalid_argument _ -> ());
  assert (List.length library = 19);
  let one_shot = List.assoc "one_shot" library in
  let pulse = advance one_shot [ (ext "one_shot" "I", true) ] in
  assert (not (value pulse));
  assert (value (advance pulse [ (ext "one_shot" "I", true) ]));
  assert (not (value (clocks 6 one_shot [ (ext "one_shot" "I", true) ])));
  let xor = List.assoc "XOR" library in
  List.iter
    (fun a ->
      List.iter
        (fun b ->
          assert (
            value (clocks 6 xor [ (ext "XOR" "IN0", a); (ext "XOR" "IN1", b) ])
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
              (clocks 12 adder [ (ext "ADDR" "I0", a); (ext "ADDR" "I1", b) ])
            = [ (ext "ADDR" "S", a <> b); (ext "ADDR" "C", a && b) ]))
        [ false; true ])
    [ false; true ];
  List.iter
    (fun name ->
      let c = clocks 20 (List.assoc name library) [] in
      assert (ok (restore (snapshot c)) = c);
      assert (List.length (as_graph_group c).verts > 0))
    [ "RS_latch"; "DFF" ];
  let iter = ok (Code.parse "iter: main {DLY_F, next {OUT=IN} prev {}}") in
  assert (
    get_inpins iter = [ ext "main" "IN" ]
    && get_otputs iter = []
    && advance iter [] = iter);
  (* Instance IDs have the same representation in input and output. *)
  let iter = advance iter [ (instance 0 "IN", true) ] in
  (match iter with
  | Iter i -> assert (state_as_gate (List.hd i.used) = Some true)
  | _ -> assert false);
  let seeded =
    advance
      (ok (Code.parse "iter: main {DLY_F, next {OUT=IN} prev {}}"))
      (ok (Rinput.parse "@main.IN=#true"))
  in
  assert (seeded = iter);
  (match iter with
  | Iter i -> assert (List.length i.used = 1)
  | _ -> assert false);
  assert (get_inpins iter = [ instance 0 "IN" ]);
  let extended = advance iter [] in
  (match extended with
  | Iter i -> assert (List.length i.used = 2)
  | _ -> assert false);
  assert (
    (as_graph_group extended).edges = [ (instance 0 "OUT", instance 1 "IN") ]);
  assert (ok (restore (snapshot extended)) = extended);
  bad (step iter [ (instance 10 "IN", true) ]);
  let backwards =
    ok (new_iter "back" (new_gate Delay false) [] [ ("OUT", "IN") ])
  in
  let backwards = advance backwards [ (instance 0 "IN", true) ] in
  bad (step backwards []);
  print_endline "logic_circuit tests passed"
