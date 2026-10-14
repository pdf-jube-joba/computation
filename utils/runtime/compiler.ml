module type S = sig
  module Source : Language.S
  module Target : Language.S

  val compile : Source.Code.t -> (Target.Code.t, string) result
  val encode_ainput : Source.Ainput.t -> (Target.Ainput.t, string) result
  val encode_rinput : Source.Rinput.t -> (Target.Rinput.t, string) result
  val decode_routput : Target.Routput.t -> (Source.Routput.t, string) result
  val decode_foutput : Target.Foutput.t -> (Source.Foutput.t, string) result
end

(** Compose typed transformations; output decoding runs in the reverse order. *)
module Compose
    (A : S)
    (B :
      S
        with type Source.Code.t = A.Target.Code.t
         and type Source.Ainput.t = A.Target.Ainput.t
         and type Source.Rinput.t = A.Target.Rinput.t
         and type Source.Routput.t = A.Target.Routput.t
         and type Source.Foutput.t = A.Target.Foutput.t) =
struct
  module Source = A.Source
  module Target = B.Target

  let compose first second input = Result.bind (first input) second
  let compile = compose A.compile B.compile
  let encode_ainput = compose A.encode_ainput B.encode_ainput
  let encode_rinput = compose A.encode_rinput B.encode_rinput
  let decode_routput = compose B.decode_routput A.decode_routput
  let decode_foutput = compose B.decode_foutput A.decode_foutput
end
