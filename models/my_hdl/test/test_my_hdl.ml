open My_hdl

let ok = function Ok x -> x | Error e -> failwith e
let bad = function Error _ -> () | Ok _ -> failwith "Expected error"
let bit b = Val (Value.Bit b)
let eval e = ok (eval_to_val [] [] e)
let env code = { code; mod_env = [] }

let () =
  assert (ok (parse_value "#true") = Value.Bit true);
  assert (
    ok (parse_value "/* nested opener /* closes at first terminator */ #true")
    = Value.Bit true);
  assert (
    ok
      (parse_value "[/* quoted \" /* } ignored */ #true // /* ignored\n #false]")
    = Value.Array [ Value.Bit true; Value.Bit false ]);
  bad (parse_value "/* nested opener /* */ */ #true");
  bad (parse_value "/* unclosed #true");
  assert (
    ok
      (Interface.parse_value
         "/* comment */ [#false, <Red>,, [#true #false], {}]")
    = Value.Array
        [
          Value.Bit false;
          Value.Enume "Red";
          Value.Array [ Value.Bit true; Value.Bit false ];
          Value.Strct [];
        ]);
  assert (ok (parse_value "<A-b_1>") = Value.Enume "A-b_1");
  assert (ok (parse_value "<->") = Value.Enume "-");
  assert (Value.new_strct [] = Some (Value.Strct []));
  assert (Value.new_strct [ ("field", Value.Bit true) ] = None);
  List.iter
    (fun text -> bad (parse_value text))
    [
      "{a: #true}";
      "{a:#true a:#false}";
      "[true]";
      "<#true>";
      "<123>";
      "#true trailing";
      "[#true";
      "{a #true}";
      "";
    ];
  let value = ok (parse_value "[#true <tag> {}]") in
  assert (
    ok
      (Value.of_yojson
         (Yojson.Safe.from_string
            (Yojson.Safe.to_string (Value.to_yojson value))))
    = value);
  assert (ok (parse_value (Value.print value)) = value);
  assert (
    Value.get_field_of_value (Value.Strct [ ("a", value) ]) "a" = Some value);
  assert (Value.get_field_of_value value "a" = None);
  assert (ok (typable_value (Value.Bit true) Value_type.Bit) = ());
  let array_type = Value_type.Array (Utils.Number.of_int 2, Value_type.Bit) in
  assert (
    ok
      (typable_value
         (Value.Array [ Value.Bit true; Value.Bit false ])
         array_type)
    = ());
  bad (typable_value (Value.Array []) array_type);
  bad
    (typable_value (Value.Array [ Value.Bit true; Value.Enume "x" ]) array_type);
  assert (
    ok
      (typable_value
         (Value.Strct [ ("b", Value.Enume "x"); ("a", Value.Bit true) ])
         (Value_type.Strct
            [ ("a", Value_type.Bit); ("b", Value_type.Enume [ "x"; "y" ]) ]))
    = ());
  bad
    (typable_value
       (Value.Strct [ ("a", Value.Bit true) ])
       (Value_type.Strct []));
  bad (typable_value (Value.Enume "x") (Value_type.Enume [ "y" ]));
  bad (typable_value (Value.Bit true) (Value_type.Enume [ "true" ]));
  (* Sets of keys and first matching values are the source's actual rules. *)
  assert (
    ok
      (typable_value
         (Value.Strct [ ("a", Value.Bit true); ("a", Value.Enume "bad") ])
         (Value_type.Strct [ ("a", Value_type.Bit) ]))
    = ());
  List.iter
    (fun a ->
      List.iter
        (fun b ->
          assert (eval (And (bit a, bit b)) = Value.Bit (a && b));
          assert (eval (Or (bit a, bit b)) = Value.Bit (a || b)))
        [ true; false ])
    [ true; false ];
  assert (eval (Not (bit false)) = Value.Bit true);
  assert (eval (IfCond (bit true, bit false, bit true)) = Value.Bit false);
  assert (eval (IfCond (bit false, bit false, bit true)) = Value.Bit true);
  assert (
    eval (ArrayAcc (ArrayConst [ bit true; bit false ], Utils.Number.one))
    = Value.Bit false);
  assert (
    eval (FieldAcc (StrctConst [ ("field", bit true) ], "field"))
    = Value.Bit true);
  assert (
    eval (Switch (Val (Value.Enume "a"), [ ("b", bit false); ("a", bit true) ]))
    = Value.Bit true);
  assert (
    eval (SeqAndFinal ([ ("x", bit true); ("y", Not (Var "x")) ], Var "y"))
    = Value.Bit false);
  assert (
    ok
      (eval_to_val []
         [ ("x", Value.Bit false) ]
         (SeqAndFinal ([ ("x", bit true) ], Var "x")))
    = Value.Bit false);
  let module_def =
    {
      name = "negate";
      input_type = Value_type.Bit;
      otput_type = Value_type.Bit;
      comb_func = Not (Var "IN");
    }
  in
  assert (
    ok (eval_to_val [ module_def ] [] (CombModule ("negate", bit true)))
    = Value.Bit false);
  let untyped = { module_def with comb_func = Var "IN" } in
  assert (
    ok
      (eval_to_val [ untyped ] []
         (CombModule ("negate", Val (Value.Enume "x"))))
    = Value.Enume "x");
  let expression =
    SeqAndFinal
      ([ ("x", ArrayConst [ bit true ]) ], ArrayAcc (Var "x", Utils.Number.zero))
  in
  assert (
    ok (comb_exp_val_of_yojson (comb_exp_val_to_yojson expression)) = expression);
  List.iter
    (fun expression -> bad (eval_to_val [] [] expression))
    [
      Var "missing";
      And (bit false, Var "missing");
      Or (bit true, Var "missing");
      And (Val (Value.Array []), bit true);
      Not (Val (Value.Enume "x"));
      IfCond (bit true, bit true, Var "missing");
      IfCond (Val (Value.Array []), bit true, bit false);
      ArrayAcc (ArrayConst [], Utils.Number.zero);
      ArrayAcc (bit false, Utils.Number.zero);
      ArrayAcc (ArrayConst [], "999999999999999999999999999999");
      FieldAcc (StrctConst [], "missing");
      FieldAcc (bit true, "x");
      Switch (Val (Value.Enume "a"), [ ("a", bit true); ("b", Var "missing") ]);
      Switch (Val (Value.Enume "missing"), []);
      Switch (bit true, []);
      CombModule ("missing", bit true);
    ];
  let isolated = { module_def with comb_func = Var "outer" } in
  bad
    (eval_to_val [ isolated ]
       [ ("outer", Value.Bit true) ]
       (CombModule ("negate", bit false)));
  let machine : simple_module_state =
    {
      state = Value.Bit false;
      trans_func = env (Or (Var "IN", Var "STATE"));
      otput_func = env (Var "STATE");
    }
  in
  assert (simple_now_state machine = Value.Bit false);
  assert (ok (simple_get_otput machine) = Value.Bit false);
  let next = ok (simple_clock machine (Value.Bit true)) in
  assert (simple_now_state next = Value.Bit true);
  assert (
    ok (simple_get_otput (ok (simple_clock next (Value.Bit false))))
    = Value.Bit true);
  bad (simple_clock machine (Value.Enume "x"));
  assert (simple_now_state machine = Value.Bit false);
  assert (
    ok (simple_module_state_of_yojson (simple_module_state_to_yojson next))
    = next);
  bad
    (build_simple
       {
         state_type_and_init = (Value.Bit false, Value_type.Bit);
         input_type = Value_type.Bit;
         otput_type = Value_type.Bit;
         trans_func = bit true;
         otput_func = bit true;
       });
  bad (general_now_state (SimpleState machine));
  bad (general_get_otput (SimpleState machine));
  bad (general_clock (SimpleState machine) (Value.Bit false));
  let graph : graph_module_state =
    {
      states_machines = [];
      trans_func = env (bit true);
      otput_func = env (bit false);
    }
  in
  assert (ok (graph_now_state graph) = Value.Strct []);
  assert (ok (graph_get_otput graph) = Value.Bit true);
  assert (ok (graph_clock graph (Value.Bit false)) = graph);
  bad
    (graph_get_otput
       { graph with states_machines = [ ("child", SimpleState machine) ] });
  let iter : iter_module_state =
    {
      init_state_machine = SimpleState machine;
      state_machines = [];
      trans_func = env (bit false);
      input_func = env (Var "IN");
      otput_func = env (bit true);
    }
  in
  bad (iter_get_otput iter);
  bad (iter_clock iter (Value.Bit false));
  print_endline "my_hdl tests passed"
