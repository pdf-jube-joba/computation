type t = string [@@deriving yojson]

let is_start = function 'a' .. 'z' | 'A' .. 'Z' | '_' -> true | _ -> false
let is_continue c = is_start c || match c with '0' .. '9' -> true | _ -> false

let valid name =
  String.length name > 0 && is_start name.[0] && String.for_all is_continue name

let make name =
  if valid name then Ok name else Error ("Invalid identifier: " ^ name)

let of_yojson = function
  | `String name -> make name
  | _ -> Error "Expected identifier string"

let to_string name = name
let compare = String.compare

module Map = Map.Make (String)
module Set = Set.Make (String)
