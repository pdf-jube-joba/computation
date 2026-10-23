module Tape = Utils_data.Tape.Identifier
module Sign = Tape.Sign
module Direction = Tape.Direction
module State_id = Utils_names.Id.Make ()

type direction = Utils_data.Tape.direction = Right | Constant | Left
[@@deriving yojson]

type sign = Sign.t [@@deriving yojson]
type tape = sign Utils_data.Tape.t [@@deriving yojson]

type code_entry =
  (sign list * State_id.t) * (sign list * State_id.t * direction list)
[@@deriving yojson]

type code = {
  tape_count : int;
  alphabet : sign list;
  init_state : State_id.t;
  accepted_state : State_id.t list;
  code : code_entry list;
  state_names : (State_id.t * Utils_names.Identifier.t) list;
}
[@@deriving yojson]

type ainput = tape list
type rinput = unit
type routput = unit
type foutput = tape list

type state = {
  definition : code;
  current_state : State_id.t;
  tapes : tape list;
}
[@@deriving yojson]
