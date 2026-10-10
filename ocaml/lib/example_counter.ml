open Machine

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
