module Source = Example_counter.Main
module Target = Example_counter.Main

let compile code = Ok code
let encode_ainput input = Ok input
let encode_rinput input = Ok input
let decode_routput output = Ok output
let decode_foutput output = Ok output

let api =
  Utils.Compiler.make_api
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
