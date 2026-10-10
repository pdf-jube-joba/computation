open Computation
open Js_of_ocaml

let export_model (module M : Machine.S) =
  let module S = Session.Make (M) in
  let session = S.create () in
  let response result =
    let json =
      match result with
      | Ok value -> `Assoc [ ("ok", value) ]
      | Error message -> `Assoc [ ("error", `String message) ]
    in
    Js.string (Yojson.Basic.to_string json)
  in
  object%js
    method make code ainput =
      response (S.make session (Js.to_string code) (Js.to_string ainput))

    method step input = response (S.step session (Js.to_string input))
    method snapshot = response (S.snapshot session)

    method restore json =
      response
        (try S.restore session (Yojson.Basic.from_string (Js.to_string json))
         with Yojson.Json_error message -> Error message)

    method render = response (S.render session)

    (* The existing assets renderer renders any saved snapshot, independently of
       the active session. This also works after the machine has halted. *)
    method renderSnapshot json =
      response
        (try
           let open Machine in
           let* state =
             M.restore (Yojson.Basic.from_string (Js.to_string json))
           in
           Ok (M.render state)
         with Yojson.Json_error message -> Error message)
  end

let () =
  let export_name =
    match Sys.backend_type with
    | Sys.Other "js_of_ocaml" -> "computationOcamlJs"
    | _ -> "computationOcamlWasm"
  in
  Js.export export_name
    object%js
      method counter = export_model (module Example_counter)
      method lambda = export_model (module Lambda_calculus)
    end
