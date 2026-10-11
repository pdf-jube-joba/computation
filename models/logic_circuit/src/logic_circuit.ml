include Circuit
module Circuit = Circuit
module Parser = Parser
module Example = Example
module Manipulation = Parser
module Parse = Parser

let parse_code = Parser.parse_main
let parse_ainput = Utils.Machine.empty_input
let parse_rinput = Parser.parse_signal
let print_code (_ : t) = "logic_circuit"
let make code () = Ok code
let snapshot = to_yojson

let restore json =
  Result.bind (of_yojson json) (fun circuit ->
      Result.map (fun () -> circuit) (validate_identifiers circuit))

let machine_step state input =
  Result.map
    (fun next -> Utils.Machine.Continue { next; output = get_otputs next })
    (Circuit.step state input)

let routput_json signal = `String (print_signal signal)
let foutput_json () = `String ""

module S = struct
  type code = t
  type ainput = unit
  type rinput = signal
  type routput = signal
  type foutput = unit
  type state = t

  let parse_code = parse_code
  let parse_ainput = parse_ainput
  let parse_rinput = parse_rinput
  let make = make
  let step = machine_step
  let snapshot = snapshot
  let restore = restore
  let render = render
  let routput_json = routput_json
  let foutput_json = foutput_json
end

module Machine = S
