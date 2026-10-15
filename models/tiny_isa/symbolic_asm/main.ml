include Types
module Code = Code
open Assembler

let make code (input : ainput) =
  Utils_base.Error.protect (fun () ->
      Tiny_isa_shared.Registers.validate input.regs;
      let insts, static_data, symbols, debug_info =
        Utils_base.Error.get (preprocess code)
      in
      let pc = Utils_base.Error.get (N.to_int input.regs.(0)) in
      {
        code;
        insts;
        static_data;
        symbols;
        debug_info;
        pc;
        regs = Array.copy input.regs;
        flag = false;
        static_data_len = List.length static_data;
        data_memory = Array.of_list (static_data @ input.data_extension);
        halted = false;
      })

let output s : asm_output =
  { regs = Array.copy s.regs; data_memory = Array.to_list s.data_memory }

let step s () =
  Utils_base.Error.protect (fun () ->
      if s.halted then Utils_runtime.Machine.Halt (output s)
      else if s.pc >= Array.length s.insts then (
        let regs = Array.copy s.regs in
        regs.(0) <- N.of_int s.pc;
        Utils_runtime.Machine.Halt (output { s with regs; halted = true }))
      else if s.pc < 0 then failwith "Negative program counter"
      else
        let regs, flag, data_memory, halted =
          Tiny_isa_shared.Execute.execute
            ~resolve:(resolve_raw_value s.symbols Text s.pc)
            s.insts.(s.pc) s.regs s.flag s.data_memory
        in
        let pc = Utils_base.Error.get (N.to_int regs.(0)) in
        let next = { s with regs; flag; data_memory; halted; pc } in
        if halted then Utils_runtime.Machine.Halt (output next)
        else Utils_runtime.Machine.Continue { next; output = () })

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      Utils_base.Error.protect (fun () ->
          Tiny_isa_shared.Registers.validate s.regs;
          if
            s.pc < 0 || s.static_data_len < 0
            || s.static_data_len > Array.length s.data_memory
          then failwith "Invalid assembly snapshot";
          s))

let render = View.render

module Ainput = struct
  type t = ainput

  let parse text =
    Result.map
      (fun (image : N.t Utils_data.Memory_image.t) ->
        { regs = image.register; data_extension = image.memory })
      (Utils_codec.Memory_image_text.Natural.parse ~register_count:8 text)

  let print =
   fun (image : t) ->
    Utils_codec.Memory_image_text.Natural.print
      { register = image.regs; memory = image.data_extension }
end

module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit

module Foutput = struct
  type t = foutput

  let parse text =
    Result.map
      (fun (image : N.t Utils_data.Memory_image.t) ->
        ({ regs = image.register; data_memory = image.memory } : asm_output))
      (Utils_codec.Memory_image_text.Natural.parse ~register_count:8 text)

  let print =
   fun (image : t) ->
    Utils_codec.Memory_image_text.Natural.print
      { register = image.regs; memory = image.data_memory }
end
