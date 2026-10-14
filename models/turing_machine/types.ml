module State_id = Utils_names.Id.Make ()
module Names = Utils_names.Naming.Make (State_id)

type direction = Right | Constant | Left [@@deriving yojson]
type sign = Utils_names.Identifier.t option [@@deriving yojson]

type tape = { left : sign list; head : sign; right : sign list }
[@@deriving yojson]

(* Both sides are stored nearest to the head first. *)
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
