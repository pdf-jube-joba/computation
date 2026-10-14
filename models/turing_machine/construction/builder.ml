open Turing_machine.Main

type 'sign user_code_entry = {
  key_sign : 'sign;
  key_state : string;
  value_sign : 'sign;
  value_state : string;
  direction : direction;
}

type 'sign t = {
  name : string;
  init_state : string;
  accepted_state : string list;
  code : 'sign user_code_entry list;
}

let new_builder name init_state =
  let open Utils_runtime.Machine in
  let* name = Utils_names.Identifier.make name in
  let* init_state = State.parse init_state in
  Ok { name; init_state; accepted_state = []; code = [] }

let of_entry ((key_sign, key_state), (value_sign, value_state, direction)) =
  { key_sign; key_state; value_sign; value_state; direction }

let to_entry ~sign e =
  ( (sign e.key_sign, e.key_state),
    (sign e.value_sign, e.value_state, e.direction) )

let build ~sign b tape =
  let open Utils_runtime.Machine in
  let* d =
    Code.make b.init_state b.accepted_state (List.map (to_entry ~sign) b.code)
  in
  make d tape
