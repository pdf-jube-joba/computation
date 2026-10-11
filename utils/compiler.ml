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

  val compile : Source.Code.t -> (Target.Code.t, string) result
  val encode_ainput : Source.Ainput.t -> (Target.Ainput.t, string) result
  val encode_rinput : Source.Rinput.t -> (Target.Rinput.t, string) result
  val decode_routput : Target.Routput.t -> (Source.Routput.t, string) result
  val decode_foutput : Target.Foutput.t -> (Source.Foutput.t, string) result
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
