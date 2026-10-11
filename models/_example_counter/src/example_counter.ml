open Utils.Machine

type code = int
type ainput = unit
type rinput = Increment | Decrement
type routput = string
type foutput = string
type state = int

let parse_code text = if String.trim text = "" then Ok 0 else natural text
let parse_ainput = empty_input

let parse_rinput text =
  match String.trim text with
  | "inc" -> Ok Increment
  | "dec" -> Ok Decrement
  | _ -> Error "Expected inc or dec"

let make code () = Ok code

let step count = function
  | Increment when count = max_int -> Error "Counter overflow"
  | Increment ->
      let next = count + 1 in
      if next >= 10 then Ok (Halt "End")
      else Ok (Continue { next; output = "inc" })
  | Decrement when count = 0 -> Error "Count cannot be negative"
  | Decrement -> Ok (Continue { next = count - 1; output = "dec" })

let snapshot count = `Assoc [ ("count", `Int count) ]

let restore = function
  | `Assoc [ ("count", `Int count) ] when count >= 0 -> Ok count
  | _ -> Error "Expected a counter snapshot with a nonnegative count"

let render count =
  let text value =
    `Assoc [ ("kind", `String "text"); ("text", `String value) ]
  in
  `List
    [
      `Assoc
        [
          ("kind", `String "table");
          ("title", `String "status");
          ("columns", `List [ text "field"; text "value" ]);
          ( "rows",
            `List
              [
                `Assoc
                  [
                    ("cells", `List [ text "count"; text (string_of_int count) ]);
                  ];
              ] );
        ];
    ]

let routput_json text = `String text
let foutput_json text = `String text

module Machine = struct
  type nonrec code = code
  type nonrec ainput = ainput
  type nonrec rinput = rinput
  type nonrec routput = routput
  type nonrec foutput = foutput
  type nonrec state = state

  let parse_code = parse_code
  let parse_ainput = parse_ainput
  let parse_rinput = parse_rinput
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
  let routput_json = routput_json
  let foutput_json = foutput_json
end

module Compiler = struct
  module Source = Machine
  module Target = Machine

  let compile code = Ok code
  let encode_ainput input = Ok input
  let encode_rinput input = Ok input
  let decode_routput output = Ok output
  let decode_foutput output = Ok output
end
