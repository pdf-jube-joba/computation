module Session = Utils_runtime.Session
module Session_json = Utils_runtime.Session_json
open Io
open Arguments
module Registry = Model_registry.Registry

let run = function
  | "model" :: name :: args ->
      let fields = options args in
      let code =
        match List.assoc_opt "--code" fields with
        | Some code -> code
        | None -> prompt "code"
      in
      let ainput =
        match List.assoc_opt "--ainput" fields with
        | Some value -> value
        | None -> prompt "ainput"
      in
      let module M = (val get (Registry.model name)) in
      let module S = Session.Make (M) in
      let session = S.create () in
      ignore (get (S.make session code ainput));
      print_endline "world=model";
      output (get (S.snapshot session));
      let rec loop () =
        print_string "rinput> ";
        flush stdout;
        match input_line stdin with
        | ":q" | ":quit" | ":exit" -> ()
        | ":snapshot" ->
            output (get (S.snapshot session));
            loop ()
        | input -> (
            let input = expand_input input in
            match S.step session input with
            | Error message ->
                prerr_endline message;
                loop ()
            | Ok result ->
                output (Session_json.step result);
                if
                  match result with
                  | Session.Halted _ -> true
                  | Session.Continued _ -> false
                then print_endline "halted"
                else (
                  output (get (S.snapshot session));
                  loop ()))
        | exception End_of_file -> ()
      in
      loop ()
  | "compiler" :: operation :: name :: args ->
      print_endline (Compiler_command.run operation name (options args))
  | args -> Pipe.run args
