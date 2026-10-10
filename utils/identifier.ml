type t = string [@@deriving yojson]

let valid name =
  let start = function 'a' .. 'z' | 'A' .. 'Z' | '_' -> true | _ -> false in
  String.length name > 0
  && start name.[0]
  && String.for_all
       (fun c ->
         start c || match c with '0' .. '9' | '-' -> true | _ -> false)
       name

let make name =
  if valid name then Ok name else Error ("Invalid identifier: " ^ name)

let of_yojson = function
  | `String name -> make name
  | _ -> Error "Expected identifier string"

let to_string name = name
let compare = String.compare

module Map = Map.Make (String)
module Set = Set.Make (String)
