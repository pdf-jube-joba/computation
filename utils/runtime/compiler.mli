module type S = sig
  module Source : Language.S
  module Target : Language.S

  val compile : Source.Code.t -> (Target.Code.t, string) result
  val encode_ainput : Source.Ainput.t -> (Target.Ainput.t, string) result
  val encode_rinput : Source.Rinput.t -> (Target.Rinput.t, string) result
  val decode_routput : Target.Routput.t -> (Source.Routput.t, string) result
  val decode_foutput : Target.Foutput.t -> (Source.Foutput.t, string) result
end

module Compose
    (A : S)
    (B :
      S
        with type Source.Code.t = A.Target.Code.t
         and type Source.Ainput.t = A.Target.Ainput.t
         and type Source.Rinput.t = A.Target.Rinput.t
         and type Source.Routput.t = A.Target.Routput.t
         and type Source.Foutput.t = A.Target.Foutput.t) :
  S with module Source = A.Source and module Target = B.Target
