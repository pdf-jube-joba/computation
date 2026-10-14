(** Wire representation shared by CLI and browser hosts. *)
let unit () = `Null

let step = function
  | Session.Continued output ->
      `Assoc [ ("kind", `String "continue"); ("output", `String output) ]
  | Session.Halted output ->
      `Assoc [ ("kind", `String "halt"); ("output", `String output) ]
