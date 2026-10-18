open Tiny_isa_shared.Instruction

type code = N.t list [@@deriving yojson]
type register_memory_image = N.t Utils_data.Memory_image.t [@@deriving yojson]
type ainput = register_memory_image
type rinput = unit
type routput = unit
type foutput = register_memory_image

type state = {
  regs : N.t Utils_data.Memory.t;
  flag : bool;
  code_len : int;
  memory : N.t Utils_data.Memory.t;
  halted : bool;
}
[@@deriving yojson]

module Code = Utils_codec.Natural_lines

let make code (input : ainput) =
  Utils_base.Error.protect (fun () ->
      Tiny_isa_shared.Registers.validate input.register;
      {
        regs = Array.copy input.register;
        flag = false;
        code_len = List.length code;
        memory = Array.of_list (code @ input.memory);
        halted = false;
      })

let output s =
  {
    Utils_data.Memory_image.register = Array.copy s.regs;
    memory =
      Array.to_list
        (Array.sub s.memory s.code_len (Array.length s.memory - s.code_len));
  }

let step s () =
  Utils_base.Error.protect (fun () ->
      if s.halted then Utils_runtime.Machine.Halt (output s)
      else
        let pc = Utils_base.Error.get (N.to_int s.regs.(0)) in
        let inst =
          Utils_base.Error.get
            (Encoding.decode
               (Utils_data.Memory.read ~default:N.zero s.memory pc))
        in
        let regs, flag, memory, halted =
          Tiny_isa_shared.Execute.execute ~resolve:Fun.id inst s.regs s.flag
            s.memory
        in
        let next = { s with regs; flag; memory; halted } in
        if halted then Utils_runtime.Machine.Halt (output next)
        else Utils_runtime.Machine.Continue { next; output = () })

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      Utils_base.Error.protect (fun () ->
          Tiny_isa_shared.Registers.validate s.regs;
          if s.code_len < 0 || s.code_len > Array.length s.memory then
            failwith "Invalid code length";
          s))

let render s =
  let pc = match N.to_int s.regs.(0) with Ok n -> Some n | _ -> None in
  let next =
    match pc with
    | Some i when i < Array.length s.memory -> (
        match Encoding.decode s.memory.(i) with
        | Ok inst -> inst_text N.to_string inst
        | _ -> "halt")
    | _ -> "halt"
  in
  let memory =
    Utils_runtime.Render.indexed_table ~title:"memory"
      ~columns:[ "addr"; "value" ] ?highlight:pc N.to_string
      (Array.to_list s.memory)
  in
  Utils_runtime.Render.state
    [
      Utils_runtime.Render.text_table ~title:"status"
        ~columns:[ "field"; "value" ]
        [
          [ "pc"; (match pc with Some n -> string_of_int n | None -> "?") ];
          [ "flag"; string_of_bool s.flag ];
          [ "next"; next ];
          [ "halted"; string_of_bool s.halted ];
        ];
      Tiny_isa_shared.View.registers s.regs;
      memory;
    ]

module Ainput = struct
  type t = ainput

  let parse = Utils_codec.Memory_image_text.Natural.parse ~register_count:8
  let print = Utils_codec.Memory_image_text.Natural.print
end

module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Ainput
