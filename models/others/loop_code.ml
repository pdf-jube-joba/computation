type t = { instructions : string array; jumps : int array }

let length code = Array.length code.instructions
let token code pc = code.instructions.(pc)
let matching code pc = code.jumps.(pc)
let tokens code = Array.to_list code.instructions
let print code = String.concat "" (tokens code)

let create ~opening ~closing ~allow_empty words =
  let instructions = Array.of_list words in
  let jumps = Array.make (Array.length instructions) (-1) in
  let rec pair pc stack =
    if pc = Array.length instructions then
      match stack with
      | [] -> Ok { instructions; jumps }
      | start :: _ ->
          Error (Printf.sprintf "Unclosed %s at instruction %d" opening start)
    else if instructions.(pc) = opening then pair (pc + 1) (pc :: stack)
    else if instructions.(pc) = closing then (
      match stack with
      | [] -> Error (Printf.sprintf "Unmatched %s at instruction %d" closing pc)
      | start :: _ when (not allow_empty) && pc = start + 1 ->
          Error "Empty loop body"
      | start :: rest ->
          jumps.(start) <- pc;
          jumps.(pc) <- start;
          pair (pc + 1) rest)
    else pair (pc + 1) stack
  in
  if (not allow_empty) && words = [] then Error "Expected a nonempty program"
  else pair 0 []
