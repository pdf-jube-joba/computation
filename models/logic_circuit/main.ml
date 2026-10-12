open Circuit
module Code = Parser.Code

let make code () = Result.map (fun () -> code) (validate_identifiers code)
let snapshot = to_yojson

let restore json =
  Result.bind (of_yojson json) (fun circuit ->
      Result.map (fun () -> circuit) (validate_identifiers circuit))

let step state input =
  Result.map
    (fun next -> Utils.Machine.Continue { next; output = get_otputs next })
    (Circuit.step state input)

module Ainput = Utils.Text.Unit
module Rinput = Circuit.Signal
module Routput = Rinput
module Foutput = Utils.Text.Unit

type state = Circuit.t

let render = Circuit.render
