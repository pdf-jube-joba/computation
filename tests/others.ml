module Machine = Utils_runtime.Machine
module B = Brainfuck.Main
module S = Ski_calculus.Main
module P = P_double_prime.Main

let get = function Ok value -> value | Error message -> failwith message
let check message condition = if not condition then failwith message

module Run (M : Machine.S) = struct
  let start code input =
    get (M.make (get (M.Code.parse code)) (get (M.Ainput.parse input)))

  let finish ?(fuel = 20000) code input =
    let rec loop fuel effects state =
      if fuel = 0 then Error "Step limit exceeded"
      else
        let saved = M.Snapshot.to_yojson state in
        let restored =
          get
            (M.Snapshot.of_yojson
               (Yojson.Safe.from_string (Yojson.Safe.to_string saved)))
        in
        check "snapshot roundtrip" (M.Snapshot.to_yojson restored = saved);
        check "render roundtrip" (M.render restored = M.render state);
        let result = M.step restored (get (M.Rinput.parse "")) in
        check "step mutated saved state" (M.Snapshot.to_yojson restored = saved);
        match result with
        | Error message -> Error message
        | Ok (Machine.Halt value) -> Ok (List.rev effects, M.Foutput.print value)
        | Ok (Machine.Continue { next; output }) ->
            loop (fuel - 1) (M.Routput.print output :: effects) next
    in
    loop fuel [] (start code input)

  let expect code input expected =
    let _, actual = get (finish code input) in
    check
      (code ^ ": expected " ^ expected ^ ", got " ^ actual)
      (actual = expected)

  let syntax_error code =
    check
      ("invalid syntax accepted: " ^ code)
      (Result.is_error (M.Code.parse code))

  let input_error input =
    check
      ("invalid input accepted: " ^ input)
      (Result.is_error (M.Ainput.parse input))

  let roundtrip code =
    let parsed = get (M.Code.parse code) in
    let printed = M.Code.print parsed in
    check "code roundtrip" (M.Code.print (get (M.Code.parse printed)) = printed)
end

module BF = Run (B)
module SKI = Run (S)
module PP = Run (P)

let replace key value = function
  | `Assoc fields ->
      `Assoc (List.map (fun (k, v) -> (k, if k = key then value else v)) fields)
  | _ -> failwith "expected object"

let brainfuck () =
  BF.expect "" "" "[]";
  BF.expect "ignored text" "" "[]";
  BF.expect "-.+." "" "[255, 0]";
  BF.expect ",.,.,." "[0, 255]" "[0, 255, 0]";
  BF.expect "+,." "" "[0]";
  BF.expect "[+[+]-]." "" "[0]";
  BF.expect "++[>++[>+<-]<-]>>." "" "[4]";
  BF.expect "+[>]+." "" "[1]";
  BF.expect
    (String.make 512 '>' ^ "+" ^ String.make 512 '<' ^ "." ^ String.make 512 '>'
   ^ ".")
    "" "[0, 1]";
  let hello =
    "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>."
  in
  let events, output = get (BF.finish hello "") in
  let bytes =
    List.init (String.length "Hello World!\n") (fun i ->
        Char.code "Hello World!\n".[i])
  in
  check "Hello World output" (output = B.Foutput.print bytes);
  check "each output byte emitted once, including the last one"
    (List.filter (( <> ) "") events = List.map string_of_int bytes);
  let events, output = get (BF.finish ",[.,]" "[72, 105]") in
  check "echo final output" (output = "[72, 105]");
  check "echo effects" (List.filter (( <> ) "") events = [ "72"; "105" ]);
  BF.roundtrip "comment + [ > - < ] .";
  List.iter BF.syntax_error [ "["; "]"; "]["; "[[]"; "[]]" ];
  List.iter BF.input_error [ "[256]"; "[-1]"; "[a]"; "[1 2]" ];
  check "left boundary accepted" (Result.is_error (BF.finish "<" ""));
  let state = BF.start "." "" in
  let snapshot = B.Snapshot.to_yojson state in
  List.iter
    (fun bad ->
      check "invalid BF snapshot accepted"
        (Result.is_error (B.Snapshot.of_yojson bad)))
    [
      `Null;
      replace "pc" (`Int (-1)) snapshot;
      replace "pc" (`Int 2) snapshot;
      replace "code" (`String "[") snapshot;
      replace "input" (`List [ `Int 256 ]) snapshot;
      replace "output_rev" (`List [ `Int (-1) ]) snapshot;
      replace "tape"
        (`Assoc [ ("left", `List []); ("head", `Int 256); ("right", `List []) ])
        snapshot;
    ];
  (* An empty loop is valid BF and must remain interruptible. *)
  check "empty BF loop did not diverge"
    (BF.finish ~fuel:30 "+[]" "" = Error "Step limit exceeded")

let ski () =
  List.iter
    (fun (code, expected) -> SKI.expect code "" expected)
    [
      ("I x", "x");
      ("K x y", "x");
      ("S f g x", "f x (g x)");
      ("SKK x", "x");
      ("S K K (K I)", "K I");
      ("K x (S I I (S I I))", "x");
      ("f (I x)", "f x");
      ("K (I x)", "K x");
      ("S (I K)", "S K");
      ("I", "I");
      ("x", "x");
      ("S K", "S K");
      ("K I x y", "y");
    ];
  let term source = get (S.Code.parse source) in
  List.iter
    (fun code -> check "compact SKI associativity" (term code = term "S K I"))
    [ "SKI"; "S KI"; "SK I" ];
  check "parentheses lost" (term "S(KI)" <> term "SKI");
  check "leftmost reduction order"
    (Option.map S.Code.print (S.reduce (term "f (I x) (I y)"))
    = Some "f x (I y)");
  check "S duplicates its third argument"
    (Option.map S.Code.print (S.reduce (term "S f g (I x)"))
    = Some "f (I x) (g (I x))");
  List.iter SKI.roundtrip [ "SKK(x)"; "S(KI)"; "x (y z)"; "I /* comment */ K" ];
  List.iter SKI.syntax_error [ ""; "()"; "(I"; "I)"; "I 1"; "I + K" ];
  SKI.input_error "x";
  check "SKI divergence did not remain stepwise"
    (SKI.finish ~fuel:50 "S I I (S I I)" "" = Error "Step limit exceeded");
  List.iter
    (fun json ->
      check "invalid SKI snapshot accepted"
        (Result.is_error (S.Snapshot.of_yojson json)))
    [
      `Null;
      `List [ `String "Var"; `String "S" ];
      `List [ `String "Var"; `String "SKI" ];
    ]

let p_double_prime () =
  PP.expect "R" "" "|0|";
  PP.expect "RRR" "2\n|1|" "|1|";
  PP.expect "λ" "" "|0|1";
  PP.expect "λR" "" "0|1|";
  PP.expect "λRλR" "" "0|0|";
  PP.expect "λRλRλR" "3\n|0|" "0|0|";
  PP.expect "R" "3\n|0|2,0" "0|2|0";
  PP.expect "(λR)" "3\n|1|" "0|0|";
  PP.expect "(λ)" "2\n1|1|0" "|0|0,0,0";
  PP.expect "(λ(λR))R" "2\n|0|" "|0|";
  (* Published predecessor example, expanded from r = λR, r' = r^(m-1), L = r'λ. *)
  let predecessor modulus =
    let r = "λR" in
    let decrement = String.concat "" (List.init (modulus - 1) (fun _ -> r)) in
    let left = decrement ^ "λ" in
    "R(R)" ^ left ^ "(" ^ decrement ^ "(" ^ left ^ "(" ^ left ^ "))" ^ decrement
    ^ left ^ ")R" ^ r
  in
  List.iter
    (fun (modulus, input, expected) ->
      let _, actual =
        get
          (PP.finish (predecessor modulus)
             (string_of_int modulus ^ "\n" ^ input))
      in
      let actual = get (P.Foutput.parse actual)
      and expected = get (P.Foutput.parse expected) in
      check "P double prime predecessor"
        (P.Tape.equal actual expected && actual.right = expected.right))
    [
      (2, "|0|1,1,1,1,1,1,1,1,0", "0|0|1,1,1,1,1,1,1,0");
      (2, "|0|1,0", "0|0|0");
      (3, "|0|1,1,2,0", "|0|1,1,1,0");
      (3, "|0|1,1,1,0", "0|0|2,2,0");
    ];
  PP.roundtrip " R ( λ R ) ";
  List.iter PP.syntax_error
    [ ""; "()"; "R()"; "("; ")"; "(R"; "R)"; "λ[]"; "L"; "lambda"; "R+" ];
  List.iter PP.input_error
    [ "1\n|0|"; "0\n|0|"; "2\n|2|"; "2\n|-1|"; "2\n|0|2"; "2" ];
  let snapshot = P.Snapshot.to_yojson (PP.start "λR" "") in
  List.iter
    (fun bad ->
      check "invalid P double prime snapshot accepted"
        (Result.is_error (P.Snapshot.of_yojson bad)))
    [
      `Null;
      replace "pc" (`Int 3) snapshot;
      replace "code" (`String "()") snapshot;
      replace "config"
        (`Assoc [ ("modulus", `Int 1); ("tape", P.Tape.to_yojson P.Tape.blank) ])
        snapshot;
    ];
  check "right boundary loop should diverge"
    (PP.finish ~fuel:30 "(R)" "2\n|1|" = Error "Step limit exceeded")

let () =
  brainfuck ();
  ski ();
  p_double_prime ();
  print_endline
    "Passed Brainfuck, SKI and P double prime evaluation, boundaries, parsing \
     and snapshots."
