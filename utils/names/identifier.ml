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

let make_exn name = Utils_base.Error.get (make name)
let to_string name = name
let equal = String.equal
let compare = String.compare

module Map = Map.Make (String)
module Set = Set.Make (String)

let check_unique ~message names =
  let rec loop seen = function
    | [] -> ()
    | name :: rest ->
        if Set.mem name seen then failwith (message name);
        loop (Set.add name seen) rest
  in
  loop Set.empty names

let fresh ?(prefix = "_") used =
  if not (valid (prefix ^ "0")) then invalid_arg "Invalid identifier prefix";
  let rec loop i =
    let name = prefix ^ string_of_int i in
    if Set.mem name used then loop (i + 1) else name
  in
  loop 0
