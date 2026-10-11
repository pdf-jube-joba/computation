module C = Async_await.Coroutine
module N = Utils.Number

let ok = function Ok v -> v | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "Expected error"

let next s agent =
  match ok (C.step s agent) with
  | Utils.Machine.Continue { next; output } -> (next, output)
  | _ -> failwith "Expected continue"

let rec run fuel s outputs =
  if fuel = 0 then failwith "Coroutine did not halt"
  else
    match ok (C.step s 0) with
    | Utils.Machine.Halt env -> (env, List.rev outputs)
    | Continue { next; output } -> run (fuel - 1) next (output :: outputs)

let () =
  let (_ : (module Utils.Machine.S)) = (module C.Machine) in
  let text =
    "fn main { run child -> $t; if done $t goto +2; yield; x <- 1 }\n\
     fn child { y <- 1 }"
  in
  let code = ok (C.parse_code text) in
  assert (ok (C.parse_code (C.print code)) = code);
  let env, trace = run 60 (ok (C.make code 1)) [] in
  assert (C.get_var env "x" = N.one);
  assert (C.get_var env "y" = N.one);
  assert (
    trace
    = [
        "agent 0: dequeue task 0";
        "agent 0: run child -> $t";
        "agent 0: if done $t goto +2";
        "agent 0: yield";
        "agent 0: dequeue task 1";
        "agent 0: y <- 1";
        "agent 0: return from child";
        "agent 0: release task 1";
        "agent 0: dequeue task 0";
        "agent 0: x <- 1";
        "agent 0: return from main";
        "agent 0: release task 0";
      ]);
  let code =
    ok
      (C.parse_code
         "// comment\n\
          fn main { x <- 2 + 3 * 4; call inc; y <- 2 - 9; if !done $t || x == \
          15 goto +2; yield }\n\
          fn inc { x <- x + 1 }")
  in
  (* Assignment precedence and stack calls can be tested without evaluating the unknown task id. *)
  let s = ok (C.make code 1) in
  let s, _ = next s 0 in
  let s, _ = next s 0 in
  assert (C.get_var s.env "x" = "14");
  let original = C.snapshot s in
  let s2, out = next s 0 in
  assert (out = "agent 0: call inc");
  assert (Array.length s2.tasks = 1 && List.length s2.tasks.(0) = 2);
  assert (C.snapshot s = original);
  let restored =
    ok
      (C.restore
         (Yojson.Safe.from_string (Yojson.Safe.to_string (C.snapshot s2))))
  in
  let a, oa = next s2 0 and b, ob = next restored 0 in
  assert (oa = ob && C.snapshot a = C.snapshot b);
  assert (C.get_var a.env "x" = "15");
  let code =
    ok
      (C.parse_code
         "fn main { x <- 2; if x == 2 && (x != 3 || !x < 9) goto +2; x <- 9; y \
          <- 5 * (4 + 2) - 99 }")
  in
  let env, _ = run 30 (ok (C.make code 1)) [] in
  assert (C.get_var env "x" = "2");
  assert (C.get_var env "y" = "0");
  let code =
    ok (C.parse_code "fn main { x <- 0; x <- x + 1; if x < 3 goto -1 }")
  in
  let env, _ = run 40 (ok (C.make code 1)) [] in
  assert (C.get_var env "x" = "3");
  let code =
    ok (C.parse_code "fn main { if 1 == 1 || done $missing goto +2; x <- 9 }")
  in
  let env, _ = run 20 (ok (C.make code 1)) [] in
  assert (env.vars = []);
  let code = ok (C.parse_code "fn main { x <- 1 }") in
  let s = ok (C.make code 2) in
  let s, _ = next s 0 in
  assert (
    match ok (C.step s 1) with
    | Utils.Machine.Halt env -> env.vars = []
    | _ -> false);
  error (C.step s 2);
  error (C.step s (-1));
  let env = ok (C.Foutput.parse "z = 2\na = 1\n$t = NONE\n$u = 4\na = 3") in
  assert (C.Foutput.print env = "a = 3\nz = 2\n$t = none\n$u = 4\n");
  assert (ok (C.Foutput.parse (C.Foutput.print env)) = env);
  error (C.make code 0);
  error (C.make (ok (C.parse_code "fn other {}")) 1);
  error (C.make (ok (C.parse_code "fn main {}\nfn main {}")) 1);
  List.iter
    (fun text ->
      let s = ok (C.make (ok (C.parse_code text)) 1) in
      let s, _ = next s 0 in
      error (C.step s 0))
    [
      "fn main { call missing }";
      "fn main { run missing -> $id }";
      "fn main { goto -1 }";
      "fn main { if done $missing goto +1 }";
    ];
  let s = ok (C.make (ok (C.parse_code "fn main { goto +5 }")) 1) in
  let s, _ = next s 0 in
  let s, _ = next s 0 in
  error (C.step s 0);
  List.iter
    (fun text -> error (C.parse_code text))
    [
      "";
      "fn main {";
      "fn main { run child -> t }";
      "fn main { x <- -1 }";
      "fn main { x <- 1 + }";
    ];
  assert (match C.render s2 with `List [ _; _; _; _; _ ] -> true | _ -> false);
  let code =
    ok
      (C.parse_code
         "fn main { foo_bar <- 7; call child_task; run child_task -> $task_id; \
          yield }\n\
          fn child_task { result <- 3 }")
  in
  assert (ok (C.parse_code (C.print code)) = code);
  let env, _ = run 40 (ok (C.make code 1)) [] in
  assert (C.get_var env "foo_bar" = "7" && C.get_var env "result" = "3");
  assert (List.assoc "task_id" env.task_ids = Some 1);
  assert (C.get_var (ok (C.Foutput.parse "foo_bar = +7")) "foo_bar" = "7");
  print_endline "coroutine tests passed"
