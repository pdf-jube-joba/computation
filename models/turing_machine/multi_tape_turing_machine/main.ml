include Types
module Code = Code
open Utils_runtime.Machine

module Ainput = struct
  type t = tape list

  let parse text =
    Utils_base.Error.protect (fun () ->
        let lines = String.split_on_char '\n' (String.trim text) in
        if lines = [ "" ] then failwith "Expected at least one tape";
        List.map (fun line -> Utils_base.Error.get (Tape.parse line)) lines)

  let print tapes = String.concat "\n" (List.map Tape.print tapes)
end

module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Ainput

let validate_tapes definition tapes =
  Utils_base.Error.protect (fun () ->
      if List.length tapes <> definition.tape_count then
        failwith "Input tape count differs from code";
      List.iter
        (fun tape ->
          let signs, _ = Tape.into_vec tape in
          List.iter
            (fun sign ->
              if not (List.mem sign definition.alphabet) then
                failwith ("Input symbol outside alphabet: " ^ Sign.print sign))
            signs)
        tapes)

let make definition tapes =
  let* () = Code.validate definition in
  let* () = validate_tapes definition tapes in
  Ok { definition; current_state = definition.init_state; tapes }

let next_code s =
  if List.mem s.current_state s.definition.accepted_state then None
  else
    Code.find_transition s.definition
      (List.map (fun (t : tape) -> t.head) s.tapes, s.current_state)

let one_step s =
  match next_code s with
  | None -> s
  | Some (_, (writes, current_state, dirs)) ->
      let tapes =
        List.map2
          (fun tape (sign, dir) -> Tape.move_to (Tape.head_write tape sign) dir)
          s.tapes (List.combine writes dirs)
      in
      { s with current_state; tapes }

let step s () =
  let next = one_step s in
  if next_code next = None then Ok (Halt next.tapes)
  else Ok (Continue { next; output = () })

let snapshot = state_to_yojson

let restore json =
  let* s = state_of_yojson json in
  let* () = Code.validate s.definition in
  let* () = validate_tapes s.definition s.tapes in
  if List.mem_assoc s.current_state s.definition.state_names then Ok s
  else Error "Unknown current state ID"

let render s =
  let module R = Utils_runtime.Render in
  let name id =
    Utils_names.Identifier.to_string (Code.state_name s.definition id)
  in
  let vector print xs = String.concat " " (List.map print xs) in
  R.state
    (R.text_table ~title:"code"
       ?highlight:(Option.map fst (next_code s))
       ~columns:[ "read"; "state"; "write"; "next_state"; "directions" ]
       (List.map
          (fun ((reads, source), (writes, target, dirs)) ->
            [
              vector Sign.print reads;
              name source;
              vector Sign.print writes;
              name target;
              vector Direction.print dirs;
            ])
          s.definition.code)
    :: R.text ~title:"state" (name s.current_state)
    :: List.mapi
         (fun i tape ->
           let signs, head = Tape.into_vec tape in
           R.highlight
             ~title:(Printf.sprintf "tape %d" (i + 1))
             ~index:head Sign.print signs)
         s.tapes)
