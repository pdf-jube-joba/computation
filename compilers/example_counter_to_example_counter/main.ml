module Source = Example_counter.Main
module Target = Example_counter.Main

let compile code = Ok code
let encode_ainput input = Ok input
let encode_rinput input = Ok input
let decode_routput output = Ok output
let decode_foutput output = Ok output

let api =
  Utils_runtime.Compiler_text_api.make
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
