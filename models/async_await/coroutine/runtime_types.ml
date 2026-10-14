include Syntax
include Identity

type global_env = {
  vars : (Utils_names.Identifier.t * N.t) list;
  task_ids : (Utils_names.Identifier.t * Task_id.t option) list;
}
[@@deriving yojson]

type frame = { function_name : Utils_names.Identifier.t; pc : int }
[@@deriving yojson]

type ainput = int
type rinput = int
type routput = string
type foutput = global_env

type state = {
  code : code;
  env : global_env;
  workers : Task_id.t option array;
  queue : Task_id.t list;
  tasks : frame list array;
}
[@@deriving yojson]
