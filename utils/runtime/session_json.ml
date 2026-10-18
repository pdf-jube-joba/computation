(** Wire representation shared by CLI and browser hosts. *)
let unit () = `Null

let step = function
  | Session.Continued output ->
      `Assoc [ ("kind", `String "continue"); ("output", `String output) ]
  | Session.Halted output ->
      `Assoc [ ("kind", `String "halt"); ("output", `String output) ]

module Make (M : Machine.S) = struct
  module S = Session.Make (M)
  include S

  let snapshot session = Result.map M.Snapshot.to_yojson (S.snapshot session)

  let restore session json =
    Result.bind (M.Snapshot.of_yojson json) (S.restore session)

  let render session = Result.map Render.to_yojson (S.render session)
end
