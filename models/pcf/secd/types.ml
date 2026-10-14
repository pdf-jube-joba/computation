open Pcf_shared.Syntax

type instr =
  | PushNat of N.t
  | PushBool of bool
  | PushUnit
  | Print of N.t
  | Acc of Utils_names.Identifier.t
  | MkCls of Utils_names.Identifier.t * instr list
  | MkRec of Utils_names.Identifier.t * Utils_names.Identifier.t * instr list
  | BinOp of binop
  | UnOp of unop
  | Ap
  | Ret
  | If of instr list * instr list
[@@deriving yojson]

type value =
  | VNat of N.t
  | VBool of bool
  | VUnit
  | VFun of Utils_names.Identifier.t * instr list * env
  | VRec of
      Utils_names.Identifier.t * Utils_names.Identifier.t * instr list * env

and env = (Utils_names.Identifier.t * value) list [@@deriving yojson]

type dump_frame = { stack : value list; env : env; control : instr list }
[@@deriving yojson]

type code = instr list [@@deriving yojson]
type ainput = N.t list
type rinput = unit
type routput = print_effect
type foutput = N.t

type state = {
  code : code;
  stack : value list;
  env : env;
  control : instr list;
  dump : dump_frame list;
}
[@@deriving yojson]
