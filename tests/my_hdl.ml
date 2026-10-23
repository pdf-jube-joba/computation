module H = My_hdl.Main
module P = My_hdl.Program
module Identifier = Utils_names.Identifier
open H

let get = Utils_base.Error.get
let name = Identifier.make_exn
let bit b = Val (Value.Bit b)
let var s = Var (name s)
let call s e = CombModule (name s, e)
let check message condition = if not condition then failwith message

let contains text fragment =
  let rec scan i =
    i + String.length fragment <= String.length text
    && (Utils_base.String_ext.starts_at text i fragment || scan (i + 1))
  in
  scan 0

let expect_error fragment = function
  | Error message ->
      check ("Unexpected error: " ^ message) (contains message fragment)
  | Ok _ -> failwith ("Expected error: " ^ fragment)

let declaration n body : comb_module =
  {
    name = name n;
    input_type = Value_type.Bit;
    otput_type = Value_type.Bit;
    comb_func = body;
  }

let program ?(modules = []) ?(inputs = []) code =
  get (P.make ~modules ~inputs:(List.map name inputs) code)

let eval program inputs =
  get (P.eval program (List.map (fun (n, v) -> (name n, Value.Bit v)) inputs))

let declarations () =
  let modules =
    [
      declaration "outer" (call "inner" (var "IN"));
      declaration "inner" (Not (var "IN"));
    ]
  in
  let p =
    program ~modules ~inputs:[ "outer" ]
      (ArrayConst [ call "outer" (var "outer"); call "outer" (bit false) ])
  in
  check "forward references and separate variable/module namespaces"
    (eval p [ ("outer", true) ]
    = Value.Array [ Value.Bit false; Value.Bit true ]);
  check "compiled declarations can be reused with new inputs"
    (eval p [ ("outer", false) ]
    = Value.Array [ Value.Bit true; Value.Bit true ]);
  expect_error "Duplicate module: inner"
    (P.make
       ~modules:[ List.nth modules 1; List.nth modules 1 ]
       ~inputs:[] (bit true));
  expect_error "Unknown module: absent"
    (P.make ~modules:[] ~inputs:[] (call "absent" (bit true)));
  expect_error "Module unused: Unknown module: absent"
    (P.make
       ~modules:[ declaration "unused" (call "absent" (var "IN")) ]
       ~inputs:[] (bit true));
  expect_error "Combinational module cycle"
    (P.make
       ~modules:[ declaration "self" (call "self" (var "IN")) ]
       ~inputs:[] (bit true));
  expect_error "Combinational module cycle"
    (P.make
       ~modules:
         [
           declaration "a" (call "b" (var "IN"));
           declaration "b" (call "a" (var "IN"));
         ]
       ~inputs:[] (bit true))

let scopes () =
  let p =
    program ~inputs:[ "x" ]
      (SeqAndFinal ([ (name "x", bit false); (name "y", var "x") ], var "y"))
  in
  check "sequence preserves the first binding"
    (eval p [ ("x", true) ] = Value.Bit true);
  let p =
    program
      (SeqAndFinal ([ (name "x", bit true); (name "x", bit false) ], var "x"))
  in
  check "repeated local binding preserves the first value"
    (eval p [] = Value.Bit true);
  expect_error "Unknown variable: y"
    (P.make ~modules:[] ~inputs:[]
       (SeqAndFinal ([ (name "x", var "y"); (name "y", bit true) ], var "x")));
  expect_error "Unknown variable: x"
    (P.make ~modules:[] ~inputs:[]
       (ArrayConst [ SeqAndFinal ([ (name "x", bit true) ], var "x"); var "x" ]));
  expect_error "Module isolated: Unknown variable: x"
    (P.make
       ~modules:[ declaration "isolated" (var "x") ]
       ~inputs:[ name "x" ]
       (call "isolated" (var "x")));
  expect_error "Duplicate input: x"
    (P.make ~modules:[] ~inputs:[ name "x"; name "x" ] (var "x"));
  expect_error "Missing input: x"
    (P.eval (program ~inputs:[ "x" ] (var "x")) []);
  check "field names and enum tags need no variable declarations"
    (eval
       (program
          (FieldAcc
             ( StrctConst
                 [
                   ( name "tag",
                     Switch
                       ( Val (Value.Enume (name "tag")),
                         [ (name "tag", bit true) ] ) );
                 ],
               name "tag" )))
       []
    = Value.Bit true)

let evaluation () =
  check "named convenience API uses resolved evaluation"
    (get
       (H.eval_to_val
          [ declaration "negate" (Not (var "IN")) ]
          []
          (call "negate" (bit false)))
    = Value.Bit true);
  let bad = Not (Val (Value.Array [])) in
  expect_error "expression is not boolean"
    (P.eval (program (IfCond (bit true, bit true, bad))) []);
  expect_error "expression is not boolean"
    (P.eval (program (And (bit false, bad))) []);
  expect_error "expression is not boolean"
    (P.eval
       (program
          (Switch
             ( Val (Value.Enume (name "ok")),
               [ (name "ok", bit true); (name "other", bad) ] )))
       []);
  check "independent programs cannot exchange definition tables"
    (eval
       (program ~modules:[ declaration "m" (bit true) ] (call "m" (bit false)))
       []
     = Value.Bit true
    && eval
         (program
            ~modules:[ declaration "m" (bit false) ]
            (call "m" (bit true)))
         []
       = Value.Bit false)

let snapshots () =
  let p =
    program
      ~modules:[ declaration "m" (Not (var "IN")) ]
      ~inputs:[ "x" ]
      (ArrayConst [ call "m" (var "x"); call "m" (bit false) ])
  in
  let json = P.to_yojson p in
  let restored =
    get (P.of_yojson (Yojson.Safe.from_string (Yojson.Safe.to_string json)))
  in
  check "restored program preserves sharing and behavior"
    (eval restored [ ("x", true) ] = eval p [ ("x", true) ]);
  check "snapshot is stable" (P.to_yojson restored = json);
  let fields = match json with `Assoc fields -> fields | _ -> assert false in
  check "shared module is declared once in a snapshot"
    (match List.assoc "mod_env" fields with `List [ _ ] -> true | _ -> false);
  expect_error "Unknown variable: missing"
    (P.of_yojson
       (`Assoc
          (("code", H.comb_exp_val_to_yojson (var "missing"))
          :: List.remove_assoc "code" fields)));
  let trans_func = program ~inputs:[ "IN"; "STATE" ] (Not (var "STATE")) in
  let otput_func = program ~inputs:[ "STATE" ] (var "STATE") in
  let machine : H.simple_module_state =
    { state = Value.Bit true; trans_func; otput_func }
  in
  let machine =
    get
      (H.simple_module_state_of_yojson
         (H.simple_module_state_to_yojson machine))
  in
  let next = get (H.simple_clock machine (Value.Bit false)) in
  check "clock uses the stored resolved program"
    (get (H.simple_get_otput next) = Value.Bit false);
  check "clock preserves the previous machine"
    (get (H.simple_get_otput machine) = Value.Bit true)

let () =
  declarations ();
  scopes ();
  evaluation ();
  snapshots ();
  print_endline
    "Passed HDL declaration collection, resolution and evaluation checks."
