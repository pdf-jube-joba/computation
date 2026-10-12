type result = { stdout : string; stderr : string }

let with_temp_file suffix f =
  let path = Filename.temp_file "computation-cli-test-" suffix in
  Fun.protect
    ~finally:(fun () -> Sys.remove path)
    (fun () ->
      let fd = Unix.openfile path [ Unix.O_RDWR ] 0o600 in
      Fun.protect ~finally:(fun () -> Unix.close fd) (fun () -> f path fd))

let rec waitpid pid =
  try snd (Unix.waitpid [] pid)
  with Unix.Unix_error (Unix.EINTR, _, _) -> waitpid pid

let run executable ?(input = "") ?(expected_status = 0) args =
  (* Files keep stdout and stderr separate without blocking on full pipes. *)
  with_temp_file ".stdin" (fun input_path stdin ->
      Out_channel.with_open_bin input_path (fun channel ->
          output_string channel input);
      with_temp_file ".stdout" (fun output_path stdout ->
          with_temp_file ".stderr" (fun error_path stderr ->
              let argv = Array.of_list (executable :: args) in
              let pid =
                Unix.create_process executable argv stdin stdout stderr
              in
              let status = waitpid pid in
              let read path =
                In_channel.with_open_bin path In_channel.input_all
              in
              let result =
                { stdout = read output_path; stderr = read error_path }
              in
              (match status with
              | Unix.WEXITED code when code = expected_status -> ()
              | status ->
                  let status =
                    match status with
                    | Unix.WEXITED code -> Printf.sprintf "exit %d" code
                    | Unix.WSIGNALED signal -> Printf.sprintf "signal %d" signal
                    | Unix.WSTOPPED signal -> Printf.sprintf "stopped %d" signal
                  in
                  failwith
                    (Printf.sprintf "%s: expected exit %d, got %s\n%s"
                       (String.concat " " args) expected_status status
                       result.stderr));
              result)))

let assert_contains text expected =
  try ignore (Str.search_forward (Str.regexp_string expected) text 0)
  with Not_found ->
    failwith (Printf.sprintf "Expected %S in %S" expected text)

let json = Yojson.Safe.from_string

let () =
  let executable = Sys.argv.(1) in
  let executable =
    if Filename.is_relative executable then
      Filename.concat (Sys.getcwd ()) executable
    else executable
  in
  let run = run executable in
  let open Yojson.Safe.Util in
  let manifest = json (run [ "list"; "--json" ]).stdout |> to_list in
  assert (
    List.exists
      (fun entry ->
        member "name" entry = `String "lambda_calculus"
        && member "kind" entry = `String "model")
      manifest);
  let model = [ "model"; "example_counter" ] in
  let created = (run (model @ [ "create"; "--code"; "8" ])).stdout in
  assert (json created = `Assoc [ ("count", `Int 8) ]);
  let continued = run ~input:created (model @ [ "step"; "--rinput"; "inc" ]) in
  assert (json continued.stdout = `Assoc [ ("count", `Int 9) ]);
  assert (json continued.stderr = `String "inc");
  let halted =
    run ~input:continued.stdout (model @ [ "step"; "--rinput"; "inc" ])
  in
  assert (halted.stdout = "");
  assert (json halted.stderr = `String "End");
  let failed =
    run ~input:created ~expected_status:1 (model @ [ "step"; "--rinput"; "bad" ])
  in
  assert (failed.stdout = "");
  assert_contains failed.stderr "Expected inc or dec";
  let rendered = json (run ~input:created (model @ [ "render" ])).stdout in
  let cell =
    rendered |> to_list |> List.hd |> member "rows" |> to_list |> List.hd
    |> member "cells" |> to_list
    |> fun cells -> List.nth cells 1
  in
  assert (member "text" cell = `String "8");
  let compiler = "recursive_function-goto_lang" in
  let code =
    (run [ "compiler"; "compile-code"; compiler; "--code"; "SUCC" ]).stdout
  in
  let input =
    (run [ "compiler"; "compile-ainput"; compiler; "--ainput"; "(2)" ]).stdout
  in
  let state =
    (run [ "model"; "goto_lang"; "create"; "--code"; code; "--ainput"; input ])
      .stdout
  in
  let rec finish fuel state =
    if fuel = 0 then
      failwith "Compiled GOTO program did not halt within 100 steps";
    let next =
      run ~input:state [ "model"; "goto_lang"; "step"; "--rinput"; "" ]
    in
    if next.stdout = "" then json next.stderr |> to_string
    else finish (fuel - 1) next.stdout
  in
  let output = finish 100 state in
  assert (
    String.trim
      (run [ "compiler"; "decode-foutput"; compiler; "--foutput"; output ])
        .stdout
    = "3");
  let repl =
    run ~input:"bad\ninc\ninc\n"
      [ "repl"; "model"; "example_counter"; "--code"; "8"; "--ainput"; "" ]
  in
  assert_contains repl.stderr "Expected inc or dec";
  assert_contains repl.stdout "\"count\":9";
  assert_contains repl.stdout "halted";
  let multiline =
    run ~input:":begin\n(\\x.\nx) (\\y. y)\n:end\n\n0\n"
      [ "repl"; "model"; "lambda_calculus" ]
  in
  assert_contains multiline.stdout "halted";
  print_endline
    "Native CLI pipe, compiler, error recovery, and multiline REPL tests passed"
