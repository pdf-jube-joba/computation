module Tape = Utils_codec.Tape_text.Identifier
module Sign = Tape.Sign
module Direction = Tape.Direction
module State_id = Utils_names.Id.Make ()
module Names = Utils_names.Naming.Make (State_id)

type direction = Utils_data.Tape.direction = Right | Constant | Left
[@@deriving yojson]

type sign = Sign.t [@@deriving yojson]
type tape = sign Utils_data.Tape.t [@@deriving yojson]

type code_entry = (sign * State_id.t) * (sign * State_id.t * direction)
[@@deriving yojson]

type code = {
  init_state : State_id.t;
  accepted_state : State_id.t list;
  code : code_entry list;
  state_names : (State_id.t * Utils_names.Identifier.t) list;
}
[@@deriving yojson]

type ainput = tape
type rinput = unit
type routput = unit
type foutput = tape

type state = { definition : code; current_state : State_id.t; tape : tape }
[@@deriving yojson]
