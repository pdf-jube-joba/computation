module Natural = struct
  type t = int

  let default = 0
  let equal = Int.equal
  let parse = Utils_data.Text.Natural_int.parse
  let print = string_of_int
  let to_yojson value = `Int value

  let of_yojson = function
    | `Int value when value >= 0 -> Ok value
    | _ -> Error "Expected a nonnegative cell value"
end

module Byte = struct
  include Natural

  let valid n = n >= 0 && n <= 255

  let validate value =
    if valid value then Ok value else Error "Expected a byte (0..255)"

  let parse text = Result.bind (Natural.parse text) validate
  let of_yojson json = Result.bind (Natural.of_yojson json) validate
end

module Natural_tape = Utils_data.Tape.Make (Natural)
module Byte_tape = Utils_data.Tape.Make (Byte)

module Bytes = struct
  include Utils_data.Value_list.Make (Byte)

  let parse text = if String.trim text = "" then Ok [] else parse text
end
