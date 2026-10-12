open Utils.Machine

type code = int
type ainput = unit
type rinput = Increment | Decrement
type routput = string
type foutput = string
type state = int

module Code = struct
  include Utils.Text.Natural_int

  let parse text =
    if String.trim text = "" then Ok 0 else Utils.Text.Natural_int.parse text
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

let snapshot count = `Assoc [ ("count", `Int count) ]

let restore = function
  | `Assoc [ ("count", `Int count) ] when count >= 0 -> Ok count
  | _ -> Error "Expected a counter snapshot with a nonnegative count"

let render count =
  Utils.Render.state
    [
      Utils.Render.assoc_table ~title:"status" ~columns:[ "field"; "value" ]
        string_of_int
        [ ("count", count) ];
    ]

module Ainput = Utils.Text.Unit

module Rinput = struct
  type t = rinput

  let parse text =
    match String.trim text with
    | "inc" -> Ok Increment
    | "dec" -> Ok Decrement
    | _ -> Error "Expected inc or dec"

  let print = function Increment -> "inc" | Decrement -> "dec"
end

module Routput = Utils.Text.String
module Foutput = Utils.Text.String
