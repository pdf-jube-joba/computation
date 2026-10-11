type t = T | F [@@deriving yojson]

let of_bool = function true -> T | false -> F
let to_bool = function T -> true | F -> false
let not = function T -> F | F -> T
let and_ a b = of_bool (to_bool a && to_bool b)
let or_ a b = of_bool (to_bool a || to_bool b)

let parse =
  Lexer.parse (fun p ->
      match Lexer.keyword p with
      | "true" -> T
      | "false" -> F
      | _ -> Lexer.error p "Expected boolean keyword")

let print = function T -> "\\true" | F -> "\\false"
