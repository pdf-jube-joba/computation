open Types

type t = sign

let blank = None

let parse text =
  let text = String.trim text in
  if text = "-" then Ok None
  else Result.map (fun s -> Some s) (Utils_names.Identifier.make text)

let print = function
  | None -> "-"
  | Some s -> Utils_names.Identifier.to_string s
