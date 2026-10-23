open Utils_runtime.Machine

type code = int
type ainput = unit
type rinput = Increment | Decrement
type routput = string
type foutput = string
type state = int

module Code = struct
  include Utils_data.Text.Natural_int

  let parse text =
    if String.trim text = "" then Ok 0
    else Utils_data.Text.Natural_int.parse text
end

let make code () = Ok code

let step count = function
  | Increment when count = max_int -> Error "Counter overflow"
  | Increment ->
      let next = count + 1 in
      if next >= 10 then Ok (Halt "End")
      else Ok (Continue { next; output = "inc" })
  | Decrement when count = 0 -> Error "Count cannot be negative"
  | Decrement -> Ok (Continue { next = count - 1; output = "dec" })

module Snapshot = struct
  type t = state

  let to_yojson count = `Assoc [ ("count", `Int count) ]

  let of_yojson = function
    | `Assoc [ ("count", `Int count) ] when count >= 0 -> Ok count
    | _ -> Error "Expected a counter snapshot with a nonnegative count"
end

let render count =
  [
    Utils_runtime.Render.assoc_table ~title:"status"
      ~columns:[ "field"; "value" ] string_of_int
      [ ("count", count) ];
  ]

module Ainput = Utils_data.Text.Unit

module Rinput = struct
  type t = rinput

  let parse =
    Utils_parsing.Parser.parse (fun p ->
        match Utils_parsing.Parser.keyword p with
        | "inc" -> Increment
        | "dec" -> Decrement
        | _ -> Utils_parsing.Parser.error p "Expected \\inc or \\dec")

  let print = function Increment -> "\\inc" | Decrement -> "\\dec"
end

module Routput = Utils_data.Text.String
module Foutput = Utils_data.Text.String
