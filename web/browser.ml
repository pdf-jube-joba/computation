open Utils
open Js_of_ocaml
module Json = Utils.Json
module Registry = Model_registry.Registry

let get = function Ok value -> value | Error message -> failwith message

let response = function
  | Ok value -> Js.string (Json.encode (`Assoc [ ("ok", value) ]))
  | Error message ->
      Js.string (Json.encode (`Assoc [ ("error", `String message) ]))

let export_model (module M : Machine.S) =
  let module S = Session.Make (M) in
  let session = S.create () in
  object%js
    method make code ainput =
      response (S.make session (Js.to_string code) (Js.to_string ainput))

    method step input = response (S.step session (Js.to_string input))
    method snapshot = response (S.snapshot session)

    method restore json =
      let result =
        let open Machine in
        let* json = Json.decode (Js.to_string json) in
        S.restore session json
      in
      response result

    method render = response (S.render session)

    method renderSnapshot json =
      let result =
        let open Machine in
        let* json = Json.decode (Js.to_string json) in
        let* state = M.restore json in
        Ok (M.render state)
      in
      response result
  end

let export_compiler (compiler : Compiler.api) =
  let call f input =
    response (Result.map (fun text -> `String text) (f (Js.to_string input)))
  in
  object%js
    method compileCode input = call compiler.compile_code input
    method encodeAinput input = call compiler.encode_ainput input
    method encodeRinput input = call compiler.encode_rinput input
    method decodeRoutput input = call compiler.decode_routput input
    method decodeFoutput input = call compiler.decode_foutput input
  end

let () =
  let export_name =
    match Sys.backend_type with
    | Sys.Other "js_of_ocaml" -> "computationOcamlJs"
    | _ -> "computationOcamlWasm"
  in
  Js.export export_name
    object%js
      method model name =
        export_model (get (Registry.model (Js.to_string name)))

      method compiler name =
        export_compiler (get (Registry.compiler (Js.to_string name)))

      method manifest = Js.string (Json.encode (Registry.manifest ()))
    end
