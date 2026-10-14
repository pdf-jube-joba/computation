open Io
open Arguments
module Registry = Model_registry.Registry

let run operation name fields =
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
