module S = Multi_tape_turing_machine.Main
module T = One_tape_turing_machine.Main
module C = Multi_tape_turing_machine_to_one_tape_turing_machine.Main

let get = Utils_base.Error.get
let check message condition = if not condition then failwith message
let id = Utils_names.Identifier.make_exn
let sign s = get (S.Sign.parse s)

module Run (M : Utils_runtime.Machine.S) = struct
  let run limit code input =
    let rec loop remaining state =
      if remaining = 0 then failwith "Step limit exceeded";
      match get (M.step state (get (M.Rinput.parse ""))) with
      | Utils_runtime.Machine.Halt output -> output
      | Utils_runtime.Machine.Continue { next; _ } -> loop (remaining - 1) next
    in
    loop limit (get (M.make code input))
end

module SR = Run (S)
module TR = Run (T)

let equal_tapes label expected actual =
  check (label ^ ": tape count") (List.length expected = List.length actual);
  List.iter2
    (fun expected actual ->
      check
        (label ^ ": " ^ S.Tape.print expected ^ " <> " ^ S.Tape.print actual)
        (S.Tape.equal expected actual))
    expected actual

let compare_execution label code input =
  let code = get (S.Code.parse (S.Code.print code)) in
  let initial = get (S.make code input) in
  let restored = get (S.restore (S.snapshot initial)) in
  check (label ^ ": snapshot") (S.snapshot initial = S.snapshot restored);
  check (label ^ ": rendering") (S.render initial = S.render restored);
  let target = get (C.compile code) in
  let target = get (T.Code.parse (T.Code.print target)) in
  let encoded = get (C.encode_ainput input) in
  equal_tapes (label ^ ": encoding") input (get (C.decode_foutput encoded));
  let expected = SR.run 1000 code input in
  let actual = TR.run 100000 target encoded |> C.decode_foutput |> get in
  equal_tapes label expected actual

let copy_code =
  "tapes: 2\n\
   alphabet: -,a,b\n\
   copy\n\
   done\n\
   a -,copy,a a,copy,R R\n\
   b -,copy,b b,copy,R R\n\
   - -,copy,- -,done,C C\n"

let examples () =
  compare_execution "copy"
    (get (S.Code.parse copy_code))
    (get (S.Ainput.parse "|a|b,a\n|-|"));
  compare_execution "initial accept"
    (get (S.Code.parse "tapes: 2\nalphabet: -,a\ndone\ndone\n"))
    (get (S.Ainput.parse "a|-|a\n|a|"));
  compare_execution "no matching vector"
    (get (S.Code.parse "tapes: 2\nalphabet: -,a,b\nq\n\na a,q,b b,q,L R\n"))
    (get (S.Ainput.parse "|a|\n|b|"));
  compare_execution "no transitions"
    (get (S.Code.parse "tapes: 1\nalphabet: -,a\nq\n\n"))
    (get (S.Ainput.parse "|a|"));
  compare_execution "halt by missing transition after a write"
    (get (S.Code.parse "tapes: 1\nalphabet: -,a\nq\n\n-,q,a,stuck,L\n"))
    [ S.Tape.blank ]

(* Build terminating programs from independent tape operations. Each state has
   a decoy that matches all but its last read, testing atomic vector matching. *)
let generated () =
  let rng = Random.State.make [| 1789 |] in
  let alphabet =
    [ None; sign "a"; sign "b"; sign "mt_left"; sign "mt0_h_B"; sign "a_b" ]
  in
  for count = 1 to 3 do
    for example = 0 to 7 do
      let input =
        List.init count (fun i ->
            if example mod 2 = 0 then S.Tape.blank
            else
              get
                (S.Tape.from_vec
                   [ sign "a"; None; sign "a_b"; sign "mt_left" ]
                   i))
      in
      let tapes = ref input in
      let entries = ref [] in
      for step = 0 to 5 do
        let reads = List.map (fun (t : S.tape) -> t.head) !tapes in
        let writes =
          List.init count (fun _ -> List.nth alphabet (Random.State.int rng 6))
        in
        let directions =
          List.init count (fun i ->
              match (example + i + if example < 3 then 0 else step) mod 3 with
              | 0 -> S.Left
              | 1 -> S.Right
              | _ -> S.Constant)
        in
        let source = id ("q" ^ string_of_int step) in
        let target = id ("q" ^ string_of_int (step + 1)) in
        let decoy =
          List.mapi
            (fun i s ->
              if i + 1 <> count then s else if s = None then sign "a" else None)
            reads
        in
        entries :=
          ( (decoy, source),
            (List.map (fun _ -> None) writes, id "bad", directions) )
          :: ((reads, source), (writes, target, directions))
          :: !entries;
        tapes :=
          List.map2
            (fun tape (write, dir) ->
              S.Tape.move_to (S.Tape.head_write tape write) dir)
            !tapes
            (List.combine writes directions)
      done;
      let code =
        get
          (S.Code.make ~tape_count:count ~alphabet (id "q0")
             [ id "q6"; id "bad" ]
             !entries)
      in
      equal_tapes "native simultaneous operations" !tapes
        (SR.run 100 code input);
      compare_execution
        (Printf.sprintf "generated %d/%d" count example)
        code input
    done
  done

let errors () =
  List.iter
    (fun text ->
      check "invalid program accepted" (Result.is_error (S.Code.parse text)))
    [
      "tapes: 0\nalphabet: -\nq\nh\n";
      "tapes: 2\nalphabet: a\nq\nh\n";
      "tapes: 2\nalphabet: -,-\nq\nh\n";
      "tapes: 2\nalphabet: -\nq\nh\n- -,q,-,h,C C\n";
      "tapes: 2\nalphabet: -\nq\nh\n- -,q,- -,h,C\n";
      "tapes: 2\nalphabet: -\nq\nh\n- -,q,a -,h,C C\n";
      "tapes: 1\nalphabet: -\nq\nh\n-,q,-,h,C,extra\n";
      "tapes: 1\nalphabet: -\nq\nh\n-,q,-,h,C\n-,q,-,q,L\n";
      "tapes: 1\nalphabet: -\nq\nh\n-,h,-,q,C\n";
    ];
  let code = get (S.Code.parse copy_code) in
  check "wrong input count" (Result.is_error (S.make code [ S.Tape.blank ]));
  check "unknown input symbol"
    (Result.is_error (S.make code (get (S.Ainput.parse "|unknown|\n|-|"))));
  check "empty input" (Result.is_error (S.Ainput.parse ""));
  check "empty encoding" (Result.is_error (C.encode_ainput []));
  check "invalid output" (Result.is_error (C.decode_foutput T.Tape.blank));
  let invalid = { code with tape_count = 0 } in
  check "compiler validation" (Result.is_error (C.compile invalid));
  let initial = get (S.make code [ S.Tape.blank; S.Tape.blank ]) in
  let bad_state = { initial with tapes = [] } in
  check "snapshot validation"
    (Result.is_error (S.restore (S.snapshot bad_state)));
  List.iter
    (fun tape ->
      check "malformed encoding"
        (Result.is_error (C.decode_foutput (get (T.Tape.parse tape)))))
    [
      "|mt_left|mt0_h_B";
      "|mt_left|mt0_p_B,mt_right";
      "|mt_left|mt0_h_B,mt0_h_B,mt_right";
      "|mt_left|mt1_h_B,mt_right";
      "|mt_left|mt0_h_B,mt1_h_B,mt0_p_B,mt_right";
      "|mt_left|mt0_h_B,mt_right,x";
    ]

let () =
  examples ();
  generated ();
  errors ();
  print_endline
    "Passed multi-tape execution, single-tape compilation and validation tests."
