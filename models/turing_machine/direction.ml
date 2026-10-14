open Types

type t = direction = Right | Constant | Left

let parse text =
  match String.trim text with
  | "L" -> Ok Left
  | "R" -> Ok Right
  | "C" -> Ok Constant
  | s -> Error ("Invalid direction: " ^ s)

let print = function Left -> "L" | Right -> "R" | Constant -> "C"
