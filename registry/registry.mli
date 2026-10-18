type implementation =
  | Model of (module Utils_runtime.Machine.S)
  | Compiler of Compiler_text_api.api

type entry = {
  name : string;
  library : string;
  implementation : implementation;
}

val entries : entry list
val model : string -> ((module Utils_runtime.Machine.S), string) result
val compiler : string -> (Compiler_text_api.api, string) result
val manifest : unit -> Yojson.Safe.t
