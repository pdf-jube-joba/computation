(** A language's external code and input/output formats, independent of
    execution. *)
module type S = sig
  module Code : Utils_data.Text.S
  module Ainput : Utils_data.Text.S
  module Rinput : Utils_data.Text.S
  module Routput : Utils_data.Text.S
  module Foutput : Utils_data.Text.S
end
