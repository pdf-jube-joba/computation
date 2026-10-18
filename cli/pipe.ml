module Session_json = Utils_runtime.Session_json
module Json = Utils_data.Json
module Session = Utils_runtime.Session
open Io
open Arguments
module Registry = Model_registry.Registry

let run = function
  | "model" :: name :: operation :: args -> (
      let module M = (val get (Registry.model name)) in
      let module S = Session_json.Make (M) in
      let session = S.create () and fields = options args in
      match operation with
      | "create" ->
          ignore
            (get
               (S.make session (require fields "--code")
                  (opt fields "--ainput" "")));
          output (get (S.snapshot session))
      | "step" -> (
          ignore (get (S.restore session (get (Json.decode (stdin_text ())))));
          let result = get (S.step session (require fields "--rinput")) in
          match result with
          | Session.Continued text ->
              prerr_endline (Json.encode (`String text));
              output (get (S.snapshot session))
          | Session.Halted text -> prerr_endline (Json.encode (`String text)))
      | "render" ->
          get (S.restore session (get (Json.decode (stdin_text ()))));
          output (get (S.render session))
      | _ -> failwith "Expected create, step or render")
  | "compiler" :: operation :: name :: args ->
      print_endline (Compiler_command.run operation name (options args))
  | [ "list" ] ->
      List.iter
        (fun (entry : Registry.entry) -> print_endline entry.name)
        Registry.entries
  | [ "list"; "--json" ] -> output (Registry.manifest ())
  | _ ->
      failwith
        "Usage: pipe-machine model NAME (create --code CODE [--ainput INPUT] | \
         step --rinput INPUT | render), compiler OP NAME --FIELD VALUE, or \
         list [--json]"
