(* Model fixtures preserve pre-refactor transitions, outputs and snapshot formats. *)
module Registry = Model_registry.Registry
module Session = Utils_runtime.Session
module Machine = Utils_runtime.Machine
module Json = Utils_codec.Json
module Identifier = Utils_names.Identifier
module Number = Utils_base.Number

let get = Utils_base.Error.get
let check message condition = if not condition then failwith message

let equal message expected actual =
  if expected <> actual then
    failwith (message ^ ": expected " ^ expected ^ ", got " ^ actual)

let fixtures = Yojson.Safe.from_file "model_fixtures.json" |> Json.list

let run_fixture fixture =
  let string key = Json.string (Json.field key fixture) in
  let name = string "name" in
  let module M = (val get (Registry.model name)) in
  let module S = Session.Make (M) in
  let session = S.create () in
  get (S.make session (string "code") (string "ainput"));
  (match
     List.assoc_opt "initial_snapshot"
       (match fixture with `Assoc xs -> xs | _ -> [])
   with
  | Some expected ->
      check
        (name ^ ": snapshot format changed")
        (expected = get (S.snapshot session))
  | None -> ());
  (* Every saved state must restore both its execution and its presentation. *)
  let roundtrip () =
    let snapshot = get (S.snapshot session) in
    let rendering = get (S.render session) in
    let restored = S.create () in
    get (S.restore restored (get (Json.decode (Json.encode snapshot))));
    check (name ^ ": snapshot roundtrip") (snapshot = get (S.snapshot restored));
    check
      (name ^ ": rendering after restore")
      (rendering = get (S.render restored));
    get (S.restore session snapshot)
  in
  let effects =
    List.map Json.string (Json.list (Json.field "outputs" fixture))
  in
  let steps = Json.int (Json.field "steps" fixture) in
  let expected_final = Json.field "final" fixture in
  for i = 0 to steps - 1 do
    roundtrip ();
    match get (S.step session (string "rinput")) with
    | Session.Continued output ->
        check (name ^ ": extra transition") (i < List.length effects);
        equal (name ^ ": running output") (List.nth effects i) output;
        if i = steps - 1 then
          check (name ^ ": expected halt") (expected_final = `Null)
    | Session.Halted output ->
        check (name ^ ": early halt") (i = steps - 1);
        equal (name ^ ": final output") (Json.string expected_final) output;
        check
          (name ^ ": session remains active after halt")
          (Result.is_error (S.snapshot session))
  done

let finish_model name code ainput rinput =
  let module M = (val get (Registry.model name)) in
  let module S = Session.Make (M) in
  let session = S.create () in
  get (S.make session code ainput);
  let rec loop remaining =
    if remaining = 0 then failwith (name ^ ": step limit exceeded");
    match get (S.step session rinput) with
    | Session.Halted output -> output
    | Session.Continued _ -> loop (remaining - 1)
  in
  loop 20000

let compiler_paths () =
  let paths =
    Yojson.Safe.from_file "compiler_fixtures.json"
    |> Json.list
    |> List.map (fun fixture ->
        let string key = Json.string (Json.field key fixture) in
        ( string "name",
          string "target",
          string "code",
          string "ainput",
          string "rinput",
          string "expected" ))
  in
  List.iter
    (fun (compiler, target, code, ainput, rinput, expected) ->
      let api = get (Registry.compiler compiler) in
      let output =
        finish_model target
          (get (api.compile_code code))
          (get (api.encode_ainput ainput))
          (get (api.encode_rinput rinput))
      in
      equal compiler expected (get (api.decode_foutput output));
      ignore (get (api.decode_routput "")))
    paths;
  List.map (fun (name, _, _, _, _, _) -> name) paths

let session_errors () =
  let module M = (val get (Registry.model "example_counter")) in
  let module S = Session.Make (M) in
  let session = S.create () in
  check "step before make" (Result.is_error (S.step session "inc"));
  get (S.make session "3" "");
  let saved = get (S.snapshot session) in
  List.iter
    (fun action ->
      check "invalid operation accepted" (Result.is_error (action ()));
      check "error changed active state" (saved = get (S.snapshot session)))
    [
      (fun () -> Result.map ignore (S.step session "invalid"));
      (fun () -> S.make session "invalid" "");
      (fun () -> S.restore session `Null);
    ];
  (* A valid parse followed by a failed transition must also keep its state. *)
  let module M = (val get (Registry.model "flow_ir")) in
  let module S = Session.Make (M) in
  let session = S.create () in
  get (S.make session ":main { :entry { %x := ld sacc #0; halt; } }" "");
  let saved = get (S.snapshot session) in
  check "invalid memory read accepted" (Result.is_error (S.step session ""));
  check "failed execution changed state" (saved = get (S.snapshot session))

let domains () =
  check "invalid identifier" (Result.is_error (Identifier.make "bad name"));
  check "invalid identifier JSON"
    (Result.is_error (Identifier.of_yojson (`String "bad name")));
  equal "canonical natural" "12"
    (Number.to_string (get (Number.of_string "00012")));
  check "negative natural" (Result.is_error (Number.of_yojson (`Int (-1))));
  let module Fresh = Utils_names.Naming.Fresh () in
  let names = Fresh.create ~reserved:[ Identifier.make_exn "label" ] () in
  equal "reserved name" "label_0"
    (Identifier.to_string (Fresh.take names (Identifier.make_exn "label")));
  equal "fresh name" "label_1"
    (Identifier.to_string (Fresh.take names (Identifier.make_exn "label")));
  let open Pcf_shared.Syntax in
  let block = Fun (Identifier.make_exn "x", Block (Skip, Nat Number.zero)) in
  check "expression accepted statement AST"
    (Result.is_error (Pcf_expr.Main.make block []));
  check "CEK accepted statement AST"
    (Result.is_error (Pcf_cek.Main.make block []));
  check "CEK compiler accepted statement AST"
    (Result.is_error (Pcf_expr_to_pcf_cek.Main.compile block))

let registry_coverage tested_compilers =
  let names =
    List.map (fun fixture -> Json.string (Json.field "name" fixture)) fixtures
  in
  let seen = Hashtbl.create 32 in
  List.iter
    (fun (entry : Registry.entry) ->
      check
        ("duplicate registry name: " ^ entry.name)
        (not (Hashtbl.mem seen entry.name));
      Hashtbl.add seen entry.name ();
      match entry.implementation with
      | Registry.Model _ ->
          check ("untested model: " ^ entry.name) (List.mem entry.name names)
      | Registry.Compiler _ ->
          check
            ("untested compiler: " ^ entry.name)
            (List.mem entry.name tested_compilers))
    Registry.entries

let () =
  List.iter run_fixture fixtures;
  let tested_compilers = compiler_paths () in
  session_errors ();
  domains ();
  registry_coverage tested_compilers;
  Printf.printf
    "Passed %d model fixtures and %d compiler paths, snapshot/renderer \
     roundtrips, session errors and domain invariants.\n"
    (List.length fixtures)
    (List.length tested_compilers)
