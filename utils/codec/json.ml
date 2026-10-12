type t = Yojson.Safe.t

let to_yojson json = json
let of_yojson json = Ok json
let encode = Yojson.Safe.to_string

let decode text =
  try Ok (Yojson.Safe.from_string text)
  with Yojson.Json_error message -> Error message

let protect f =
  try Ok (f ())
  with
  | Failure message | Invalid_argument message | Yojson.Json_error message ->
    Error message

let field name = function
  | `Assoc fields -> (
      match List.assoc_opt name fields with
      | Some value -> value
      | None -> failwith ("Missing field: " ^ name))
  | _ -> failwith "Expected a JSON object"

let string = function `String s -> s | _ -> failwith "Expected a JSON string"
let int = function `Int n -> n | _ -> failwith "Expected a JSON integer"
let bool = function `Bool b -> b | _ -> failwith "Expected a JSON boolean"
let list = function `List xs -> xs | _ -> failwith "Expected a JSON array"
