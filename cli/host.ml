open Utils

let pipe = Pipe.run
let repl = Repl.run

let main mode =
  try mode (Array.to_list Sys.argv |> List.tl)
  with Failure message | Invalid_argument message | Parser.Error message ->
    prerr_endline message;
    exit 1
