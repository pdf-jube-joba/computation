module N = Utils_base.Number
module Text = Utils_base.String_ext
open Tiny_isa_shared.Instruction

type raw_value =
  | Imm of N.t
  | Label of Utils_names.Identifier.t
  | Const of Utils_names.Identifier.t
[@@deriving yojson]

type raw_operand = Reg of int | Value of raw_value [@@deriving yojson]

type raw_inst = {
  mnemonic : string;
  cond_flag : bool;
  operands : raw_operand list;
}
[@@deriving yojson]

type raw_directive = Text | Data | Equ of Utils_names.Identifier.t * N.t
[@@deriving yojson]

type raw_item =
  | Directive of raw_directive
  | LabelDef of Utils_names.Identifier.t
  | Inst of raw_inst
  | DataValue of raw_value
[@@deriving yojson]

type code = raw_item list [@@deriving yojson]
type inst = raw_value Tiny_isa_shared.Instruction.inst [@@deriving yojson]

type asm_input = { regs : N.t Utils_data.Memory.t; data_extension : N.t list }
[@@deriving yojson]

type asm_output = { regs : N.t Utils_data.Memory.t; data_memory : N.t list }
[@@deriving yojson]
