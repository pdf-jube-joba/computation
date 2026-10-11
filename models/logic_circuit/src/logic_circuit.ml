include Circuit
module Circuit = Circuit
module Parser = Parser
module Example = Example
module Manipulation = Parser
module Parse = Parser
module Code = Parser.Code

let make code () = Ok code
let snapshot = to_yojson

let restore json =
  Result.bind (of_yojson json) (fun circuit ->
      Result.map (fun () -> circuit) (validate_identifiers circuit))

let machine_step state input =
  Result.map
    (fun next -> Utils.Machine.Continue { next; output = get_otputs next })
    (Circuit.step state input)

module Ainput = Utils.Text.Unit
module Rinput = Circuit.Signal
module Routput = Rinput
module Foutput = Utils.Text.Unit

module S = struct
  module Code = Code
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type state = t

  let make = make
  let step = machine_step
  let snapshot = snapshot
  let restore = restore
  let render = render
end

module Machine = S
