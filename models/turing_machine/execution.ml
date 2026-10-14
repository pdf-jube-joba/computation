open Types
open Utils_runtime.Machine

let next_code s =
  if List.mem s.current_state s.definition.accepted_state then None
  else Code.find_transition s.definition (s.tape.head, s.current_state)

let is_accepted s = List.mem s.current_state s.definition.accepted_state
let is_terminate s = next_code s = None

let make definition tape =
  let* () = Code.validate definition in
  Ok { definition; current_state = definition.init_state; tape }

let one_step s =
  match next_code s with
  | None -> s
  | Some (_, (head, current_state, dir)) ->
      {
        s with
        current_state;
        tape = Tape.move_to (Tape.head_write s.tape head) dir;
      }

let step s () =
  let next = one_step s in
  if is_terminate next then Ok (Halt next.tape)
  else Ok (Continue { next; output = () })

let result s = if is_terminate s then Ok s.tape else Error "not terminated"
