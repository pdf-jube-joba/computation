open Computation

let run (module M : Machine.S) code ainput =
  let module S = Session.Make (M) in
  let session = S.create () in
  let show = function
    | Ok json ->
        print_endline (Yojson.Basic.to_string json);
        true
    | Error message ->
        prerr_endline message;
        false
  in
  if show (S.make session code ainput) then begin
    ignore (show (S.render session));
    let rec loop () =
      match input_line stdin with
      | ":quit" -> ()
      | ":snapshot" ->
          ignore (show (S.snapshot session));
          loop ()
      | input -> (
          let result = S.step session input in
          ignore (show result);
          match result with
          | Ok (`Assoc fields)
            when List.assoc_opt "kind" fields = Some (`String "halt") ->
              ()
          | _ ->
              ignore (show (S.render session));
              loop ())
      | exception End_of_file -> ()
    in
    loop ()
  end
  else exit 1

let () =
  if Array.length Sys.argv < 3 || Array.length Sys.argv > 4 then begin
    prerr_endline
      "Usage: repl (counter|lambda) CODE [AINPUT]\n\
       stdin: inc/dec or redex index; :snapshot; :quit";
    exit 1
  end;
  let ainput = if Array.length Sys.argv = 4 then Sys.argv.(3) else "" in
  match Sys.argv.(1) with
  | "counter" -> run (module Example_counter) Sys.argv.(2) ainput
  | "lambda" -> run (module Lambda_calculus) Sys.argv.(2) ainput
  | _ ->
      prerr_endline "Unknown model";
      exit 1
