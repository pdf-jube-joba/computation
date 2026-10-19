include Syntax

type slice = Whole of int | Subslice of slice * N.t * N.t [@@deriving yojson]

type value =
  | Num of N.t
  | Code_label of Id.t
  | Slice_value of slice
  | Address of slice * N.t
[@@deriving yojson]

type memory = (N.t * value) list [@@deriving yojson]

type state = {
  code : code;
  block : Id.t;
  line : int;
  registers : (Id.t * value) list;
  memory : memory array;
}
[@@deriving yojson]

type ainput = memory
type foutput = memory
type rinput = string
type routput = string
