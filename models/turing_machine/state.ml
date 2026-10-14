type t = Utils_names.Identifier.t

let parse text = Utils_names.Identifier.make (String.trim text)
let print text = Utils_names.Identifier.to_string text
