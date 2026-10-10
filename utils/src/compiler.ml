(* Typed model-to-model compilers can retain Source/Target modules and expose
   this text boundary to the common browser and CLI hosts. *)
type api = {
  compile_code : string -> (string, string) result;
  encode_ainput : string -> (string, string) result;
  encode_rinput : string -> (string, string) result;
  decode_routput : string -> (string, string) result;
  decode_foutput : string -> (string, string) result;
}

module type S = sig
  module Source : Machine.S
  module Target : Machine.S

  val compile : Source.code -> (Target.code, string) result
  val encode_ainput : Source.ainput -> (Target.ainput, string) result
  val encode_rinput : Source.rinput -> (Target.rinput, string) result
  val decode_routput : Target.routput -> (Source.routput, string) result
  val decode_foutput : Target.foutput -> (Source.foutput, string) result
end

let identity text = Ok text

let identity_api =
  {
    compile_code = identity;
    encode_ainput = identity;
    encode_rinput = identity;
    decode_routput = identity;
    decode_foutput = identity;
  }
