open Circuit
module Code = Parser.Code

let make code () = Result.map (fun () -> code) (validate code)

let step state input =
  Result.map
    (fun next ->
      Utils_runtime.Machine.Continue { next; output = get_otputs next })
    (Circuit.step state input)

module Ainput = Utils_data.Text.Unit
module Rinput = Signal_text
module Routput = Rinput
module Foutput = Utils_data.Text.Unit

type state = Circuit.t

let render = View.render

module Snapshot = struct
  type t = state

  let to_yojson = to_yojson

  let of_yojson json =
    Result.bind (of_yojson json) (fun circuit ->
        Result.map (fun () -> circuit) (validate circuit))
end
