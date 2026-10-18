type api = {
  compile_code : string -> (string, string) result;
  encode_ainput : string -> (string, string) result;
  encode_rinput : string -> (string, string) result;
  decode_routput : string -> (string, string) result;
  decode_foutput : string -> (string, string) result;
}
(** Text transport adapter for a typed compiler. *)

val make : (module Utils_runtime.Compiler.S) -> api
