(** A language's external code and input/output formats, independent of
    execution. *)
module type S = sig
  module Code : Utils_codec.Text.S
  module Ainput : Utils_codec.Text.S
  module Rinput : Utils_codec.Text.S
  module Routput : Utils_codec.Text.S
  module Foutput : Utils_codec.Text.S
end
