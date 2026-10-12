open Utils
module Registry = Model_registry.Registry

let get = function Ok value -> value | Error message -> failwith message
let output json = print_endline (Json.encode json)

let stdin_text () =
  let buffer = Buffer.create 128 in
  (try
     while true do
       Buffer.add_string buffer (input_line stdin);
       Buffer.add_char buffer '\n'
     done
   with End_of_file -> ());
  Buffer.contents buffer

let rec options = function
  | [] -> []
  | key :: value :: rest when String.starts_with ~prefix:"--" key ->
      (key, value) :: options rest
  | args -> failwith ("Unexpected arguments: " ^ String.concat " " args)

let opt fields key default = Option.value ~default (List.assoc_opt key fields)

let require fields key =
  match List.assoc_opt key fields with
  | Some value -> value
  | None -> failwith ("Missing " ^ key)

let compiler operation name fields =
  let api = get (Registry.compiler name) in
  let f, key =
    match operation with
    | "compile-code" -> (api.compile_code, "--code")
    | "compile-ainput" | "encode-ainput" -> (api.encode_ainput, "--ainput")
    | "compile-rinput" | "encode-rinput" -> (api.encode_rinput, "--rinput")
    | "decode-routput" -> (api.decode_routput, "--routput")
    | "decode-foutput" -> (api.decode_foutput, "--foutput")
    | _ -> failwith ("Unknown compiler operation: " ^ operation)
  in
  get (f (require fields key))

let pipe = function
  | "model" :: name :: operation :: args -> (
      let module M = (val get (Registry.model name)) in
      let module S = Session.Make (M) in
      let session = S.create () and fields = options args in
      match operation with
      | "create" ->
          ignore
            (get
               (S.make session (require fields "--code")
                  (opt fields "--ainput" "")));
          output (get (S.snapshot session))
      | "step" ->
          ignore (get (S.restore session (get (Json.decode (stdin_text ())))));
          let result = get (S.step session (require fields "--rinput")) in
          let kind = Json.string (Json.field "kind" result) in
          prerr_endline (Json.encode (Json.field "output" result));
          if kind = "continue" then output (get (S.snapshot session))
      | "render" ->
          let state = get (M.restore (get (Json.decode (stdin_text ())))) in
          output (M.render state)
      | _ -> failwith "Expected create, step or render")
  | "compiler" :: operation :: name :: args ->
      print_endline (compiler operation name (options args))
  | [ "list" ] ->
      List.iter (fun (name, _) -> print_endline name) Registry.entries
  | [ "list"; "--json" ] -> output (Registry.manifest ())
  | _ ->
      failwith
        "Usage: pipe-machine model NAME (create --code CODE [--ainput INPUT] | \
         step --rinput INPUT | render), compiler OP NAME --FIELD VALUE, or \
         list [--json]"

let repl = function
  | "model" :: name :: args ->
      let fields = options args in
      let read label =
        print_string (label ^ "> ");
        flush stdout;
        match input_line stdin with
        | ":begin" ->
            let buffer = Buffer.create 128 in
            let rec lines () =
              match input_line stdin with
              | ":end" -> Buffer.contents buffer
              | line ->
                  Buffer.add_string buffer line;
                  Buffer.add_char buffer '\n';
                  lines ()
              | exception End_of_file -> Buffer.contents buffer
            in
            lines ()
        | line -> line
        | exception End_of_file -> ""
      in
      let code =
        match List.assoc_opt "--code" fields with
        | Some code -> code
        | None -> read "code"
      in
      let ainput =
        match List.assoc_opt "--ainput" fields with
        | Some value -> value
        | None -> read "ainput"
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
            let input =
              if input = ":begin" then (
                let b = Buffer.create 128 in
                let rec lines () =
                  match input_line stdin with
                  | ":end" -> ()
                  | line ->
                      Buffer.add_string b line;
                      Buffer.add_char b '\n';
                      lines ()
                  | exception End_of_file -> ()
                in
                lines ();
                Buffer.contents b)
              else input
            in
            match S.step session input with
            | Error message ->
                prerr_endline message;
                loop ()
            | Ok result ->
                output result;
                if Json.string (Json.field "kind" result) = "halt" then
                  print_endline "halted"
                else (
                  output (get (S.snapshot session));
                  loop ()))
        | exception End_of_file -> ()
      in
      loop ()
  | "compiler" :: operation :: name :: args ->
      print_endline (compiler operation name (options args))
  | args -> pipe args

let main mode =
  try mode (Array.to_list Sys.argv |> List.tl)
  with Failure message | Invalid_argument message | Parser.Error message ->
    prerr_endline message;
    exit 1
