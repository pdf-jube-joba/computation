module Text = Utils.String_ext
open Isa_common

type raw_value =
  | Imm of N.t
  | Label of Utils.Identifier.t
  | Const of Utils.Identifier.t
[@@deriving yojson]

type raw_operand = Reg of int | Value of raw_value [@@deriving yojson]

type raw_inst = {
  mnemonic : string;
  cond_flag : bool;
  operands : raw_operand list;
}
[@@deriving yojson]

type raw_directive = Text | Data | Equ of Utils.Identifier.t * N.t
[@@deriving yojson]

type raw_item =
  | Directive of raw_directive
  | LabelDef of Utils.Identifier.t
  | Inst of raw_inst
  | DataValue of raw_value
[@@deriving yojson]

type code = raw_item list [@@deriving yojson]
type inst = raw_value Isa_common.inst [@@deriving yojson]

type asm_input = { regs : N.t array; data_extension : N.t list }
[@@deriving yojson]

type asm_output = { regs : N.t array; data_memory : N.t list }
[@@deriving yojson]

type symbols = {
  consts : (Utils.Identifier.t * N.t) list;
  text_labels : (Utils.Identifier.t * int) list;
  data_labels : (Utils.Identifier.t * int) list;
  code_len : int;
  data_len : int;
}
[@@deriving yojson]

type debug_info = {
  text_labels_by_addr : (int * Utils.Identifier.t list) list;
  data_labels_by_addr : (int * Utils.Identifier.t list) list;
}
[@@deriving yojson]

type ainput = asm_input
type rinput = unit
type routput = unit
type foutput = asm_output

type state = {
  code : code;
  insts : inst array;
  symbols : symbols;
  debug_info : debug_info;
  regs : N.t array;
  flag : bool;
  pc : int;
  static_data_len : int;
  static_data : N.t list;
  data_memory : N.t array;
  halted : bool;
}
[@@deriving yojson]

let parse_reg token =
  let t = Text.strip_prefix "r" (Text.strip_prefix "%" (String.trim token)) in
  let n = value (Utils.Text.Natural_int.parse t) in
  if n >= 8 then
    failwith (Printf.sprintf "Register out of range (expected 0..=7): %d" n);
  n

let parse_raw_value token =
  let t = String.trim token in
  if t = "" then failwith "Empty value";
  if String.starts_with ~prefix:"#" t then
    Imm (value (number (Text.strip_prefix "#" t)))
  else if String.starts_with ~prefix:"@" t then
    let name = Text.strip_prefix "@" t in
    Label (value (Utils.Identifier.make name))
  else if String.for_all (function '0' .. '9' -> true | _ -> false) t then
    Imm (value (number t))
  else Const (value (Utils.Identifier.make t))

let parse_operand t =
  if String.starts_with ~prefix:"%" t then Reg (parse_reg t)
  else Value (parse_raw_value t)

let parse_inst_line line =
  match Text.words (String.trim (Text.strip_suffix ";" (String.trim line))) with
  | [] -> failwith "Empty instruction line"
  | m :: ops ->
      let cond_flag = String.ends_with ~suffix:".f" m in
      {
        mnemonic = (if cond_flag then Text.strip_suffix ".f" m else m);
        cond_flag;
        operands = List.map parse_operand ops;
      }

let parse_directive line =
  match Text.words line with
  | [ ".text" ] -> Text
  | [ ".data" ] -> Data
  | [ ".equ"; name; n ] ->
      Equ
        ( value (Utils.Identifier.make name),
          value (number (Text.strip_prefix "#" n)) )
  | _ -> failwith ("Unsupported directive syntax: " ^ line)

let parse_code text =
  Utils.Lexer.protect (fun () ->
      let section = ref Text in
      List.filter_map
        (fun line ->
          let line = String.trim (Text.strip_line_comment "//" line) in
          if line = "" then None
          else
            let item =
              if
                String.starts_with ~prefix:"@" line
                && String.ends_with ~suffix:":" line
              then
                let n =
                  String.trim
                    (Text.strip_suffix ":" (Text.strip_prefix "@" line))
                in
                LabelDef (value (Utils.Identifier.make n))
              else if String.starts_with ~prefix:"." line then (
                let d = parse_directive line in
                (match d with Text | Data -> section := d | Equ _ -> ());
                Directive d)
              else
                match !section with
                | Text -> Inst (parse_inst_line line)
                | Data ->
                    let line =
                      String.trim (Text.strip_suffix ";" (String.trim line))
                    in
                    if List.length (Text.words line) <> 1 then
                      failwith "Data value line must contain a single value";
                    DataValue (parse_raw_value line)
                | Equ _ -> assert false
            in
            Some item)
        (String.split_on_char '\n' text))

let raw_value_to_text = function
  | Imm n -> N.to_string n
  | Label n -> "@" ^ n
  | Const n -> n

let operand_to_text = function
  | Reg r -> "%r" ^ string_of_int r
  | Value v -> raw_value_to_text v

let raw_inst_to_text i =
  i.mnemonic
  ^ (if i.cond_flag then ".f" else "")
  ^ String.concat "" (List.map (fun op -> " " ^ operand_to_text op) i.operands)
  ^ ";"

let item_to_text = function
  | Directive Text -> ".text"
  | Directive Data -> ".data"
  | Directive (Equ (n, v)) -> ".equ " ^ n ^ " #" ^ N.to_string v
  | LabelDef n -> "@" ^ n ^ ":"
  | Inst i -> raw_inst_to_text i
  | DataValue v -> raw_value_to_text v

let print code = String.concat "\n" (List.map item_to_text code)

let lower_inst raw =
  let m = raw.mnemonic and cf = raw.cond_flag in
  let arity n =
    if List.length raw.operands <> n then
      failwith (Printf.sprintf "%s requires %d operands" m n)
  in
  let no_flag () = if cf then failwith (m ^ ".f is not supported") in
  let reg = function
    | Reg r when r >= 0 && r < 8 -> r
    | _ -> failwith "Expected register operand"
  in
  let imm = function Value v -> v | _ -> failwith "Expected value operand" in
  let one () =
    arity 1;
    List.hd raw.operands
  in
  let two () =
    arity 2;
    match raw.operands with [ a; b ] -> (a, b) | _ -> assert false
  in
  match m with
  | "nop" ->
      arity 0;
      no_flag ();
      Nop
  | "halt" ->
      arity 0;
      no_flag ();
      Halt
  | "rst" ->
      arity 0;
      no_flag ();
      Rst
  | "ld" ->
      no_flag ();
      let a, b = two () in
      Ld (reg a, reg b)
  | "st" ->
      no_flag ();
      let a, b = two () in
      St (reg a, reg b)
  | "mov" ->
      let a, b = two () in
      Mov (cf, reg a, reg b)
  | "jump" -> (
      match one () with
      | Reg r -> Mov (cf, 0, reg (Reg r))
      | v -> Ldi (cf, 0, imm v))
  | "ldi" ->
      let a, b = two () in
      Ldi (cf, reg a, imm b)
  | "add" -> (
      let a, b = two () in
      match b with
      | Reg r -> Add (cf, reg a, reg (Reg r))
      | b -> Addi (cf, reg a, imm b))
  | "sub" -> (
      let a, b = two () in
      match b with
      | Reg r -> Sub (cf, reg a, reg (Reg r))
      | b -> Subi (cf, reg a, imm b))
  | "addi" ->
      let a, b = two () in
      Addi (cf, reg a, imm b)
  | "subi" ->
      let a, b = two () in
      Subi (cf, reg a, imm b)
  | "eq" ->
      no_flag ();
      let a, b = two () in
      Eq (reg a, reg b)
  | "lt" ->
      no_flag ();
      let a, b = two () in
      Lt (reg a, reg b)
  | "gt" ->
      no_flag ();
      let a, b = two () in
      Gt (reg a, reg b)
  | _ -> failwith ("Unsupported symbolic instruction in preprocess: " ^ m)

let pass1_collect_symbols code =
  let section = ref Text in
  let symbols =
    ref
      {
        consts = [];
        text_labels = [];
        data_labels = [];
        code_len = 0;
        data_len = 0;
      }
  in
  let available name =
    if List.mem name [ "THIS"; "CODE_LEN"; "DATA_LEN" ] then
      failwith ("Reserved symbol name cannot be redefined: @" ^ name);
    let s = !symbols in
    if
      List.mem_assoc name s.consts
      || List.mem_assoc name s.text_labels
      || List.mem_assoc name s.data_labels
    then failwith ("Duplicate symbol definition: " ^ name)
  in
  List.iter
    (function
      | Directive ((Text | Data) as d) -> section := d
      | Directive (Equ (n, v)) ->
          available n;
          symbols := { !symbols with consts = !symbols.consts @ [ (n, v) ] }
      | LabelDef n ->
          available n;
          if !section = Text then
            symbols :=
              {
                !symbols with
                text_labels = !symbols.text_labels @ [ (n, !symbols.code_len) ];
              }
          else
            symbols :=
              {
                !symbols with
                data_labels = !symbols.data_labels @ [ (n, !symbols.data_len) ];
              }
      | Inst _ ->
          if !section <> Text then
            failwith "Instruction found outside .text section";
          symbols := { !symbols with code_len = !symbols.code_len + 1 }
      | DataValue _ ->
          if !section <> Data then
            failwith "Data value found outside .data section";
          symbols := { !symbols with data_len = !symbols.data_len + 1 })
    code;
  !symbols

let resolve_raw_value symbols section this_pos = function
  | Imm n -> n
  | Const n -> (
      match List.assoc_opt n symbols.consts with
      | Some n -> n
      | None -> failwith ("Unknown const reference: " ^ n))
  | Label n -> (
      match n with
      | "THIS" -> N.of_int this_pos
      | "CODE_LEN" -> N.of_int symbols.code_len
      | "DATA_LEN" -> N.of_int symbols.data_len
      | n -> (
          match List.assoc_opt n symbols.text_labels with
          | Some n -> N.of_int n
          | None -> (
              match List.assoc_opt n symbols.data_labels with
              | Some n -> N.of_int n
              | None ->
                  failwith
                    ("Unknown label reference in "
                    ^ (if section = Text then ".text" else ".data")
                    ^ ": @" ^ n))))

let build_debug_info symbols =
  let by_addr labels =
    List.fold_left
      (fun out (name, addr) ->
        let names = Option.value ~default:[] (List.assoc_opt addr out) in
        (addr, List.sort String.compare (name :: names))
        :: List.remove_assoc addr out)
      [] labels
    |> List.sort (fun (a, _) (b, _) -> Int.compare a b)
  in
  {
    text_labels_by_addr = by_addr symbols.text_labels;
    data_labels_by_addr = by_addr symbols.data_labels;
  }

let preprocess code =
  Utils.Lexer.protect (fun () ->
      let symbols = pass1_collect_symbols code in
      let section = ref Text
      and insts = ref []
      and data = ref []
      and pos = ref 0 in
      List.iter
        (function
          | Directive ((Text | Data) as d) -> section := d
          | Directive (Equ _) | LabelDef _ -> ()
          | Inst raw ->
              if !section <> Text then
                failwith "Instruction found outside .text section";
              insts := lower_inst raw :: !insts
          | DataValue v ->
              if !section <> Data then
                failwith "Data value found outside .data section";
              let n = resolve_raw_value symbols Data !pos v in
              incr pos;
              data := n :: !data)
        code;
      ( Array.of_list (List.rev !insts),
        List.rev !data,
        symbols,
        build_debug_info symbols ))

let make code (input : ainput) =
  Utils.Lexer.protect (fun () ->
      validate_regs input.regs;
      let insts, static_data, symbols, debug_info = value (preprocess code) in
      let pc = value (N.to_int input.regs.(0)) in
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
  Utils.Lexer.protect (fun () ->
      if s.halted then Utils.Machine.Halt (output s)
      else if s.pc >= Array.length s.insts then (
        let regs = Array.copy s.regs in
        regs.(0) <- N.of_int s.pc;
        Utils.Machine.Halt (output { s with regs; halted = true }))
      else if s.pc < 0 then failwith "Negative program counter"
      else
        let regs, flag, data_memory, halted =
          execute
            ~resolve:(resolve_raw_value s.symbols Text s.pc)
            s.insts.(s.pc) s.regs s.flag s.data_memory
        in
        let pc = value (N.to_int regs.(0)) in
        let next = { s with regs; flag; data_memory; halted; pc } in
        if halted then Utils.Machine.Halt (output next)
        else Utils.Machine.Continue { next; output = () })

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      Utils.Lexer.protect (fun () ->
          validate_regs s.regs;
          if
            s.pc < 0 || s.static_data_len < 0
            || s.static_data_len > Array.length s.data_memory
          then failwith "Invalid assembly snapshot";
          s))

let render s =
  let next =
    if s.pc >= 0 && s.pc < Array.length s.insts then
      inst_text raw_value_to_text s.insts.(s.pc)
    else "halt"
  in
  let data_rows =
    Array.to_list
      (Array.mapi
         (fun addr v ->
           let labels =
             Option.value ~default:[]
               (List.assoc_opt addr s.debug_info.data_labels_by_addr)
           in
           [
             string_of_int addr;
             (if labels = [] then "-" else String.concat ", " labels);
             N.to_string v;
           ])
         s.data_memory)
  in
  Utils.Render.state
    [
      Utils.Render.text_table ~title:"status" ~columns:[ "field"; "value" ]
        [
          [ "pc"; string_of_int s.pc ];
          [ "flag"; string_of_bool s.flag ];
          [ "next"; next ];
          [ "halted"; string_of_bool s.halted ];
        ];
      registers s.regs;
      Utils.Render.text_table ~title:"data"
        ~columns:[ "addr"; "label"; "value" ]
        data_rows;
    ]

module Ainput = struct
  type t = ainput

  let parse text =
    Result.map
      (fun (regs, data_extension) -> { regs; data_extension })
      (parse_image text)

  let print = fun (image : t) -> print_image image.regs image.data_extension
end

module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit

module Foutput = struct
  type t = foutput

  let parse text =
    Result.map
      (fun (regs, data_memory) -> ({ regs; data_memory } : asm_output))
      (parse_image text)

  let print = fun (image : t) -> print_image image.regs image.data_memory
end

module Machine = struct
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec code = code
  type nonrec state = state

  let parse_code = parse_code
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
