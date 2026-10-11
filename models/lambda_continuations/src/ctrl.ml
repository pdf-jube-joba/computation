include Lambda

type abct = term [@@deriving yojson]
type abct_value = value [@@deriving yojson]

let abort e = Abort e
let control e = Control e
let num_to_abct = num_to_exp
let abct_to_value = lam_to_value

let unsupported =
  "abort/control semantics are not implemented yet (unsupported in the \
   original Rust model)"

(* Rust's entire Step implementation is todo!, including ordinary AbCt terms. *)
let reduce _ = Error unsupported
let small_step = reduce

type code = term
type ainput = No_ext.ainput
type rinput = unit
type routput = unit
type foutput = value
type state = No_ext.state [@@deriving yojson]

let parse_code = parse_ctrl_code
let parse_ainput = No_ext.parse_ainput
let parse_rinput = No_ext.parse_rinput
let make = No_ext.make
let step _ () = Error unsupported
let snapshot = No_ext.snapshot
let restore = No_ext.restore
let render = No_ext.render
let routput_json = No_ext.routput_json
let foutput_json = No_ext.foutput_json
