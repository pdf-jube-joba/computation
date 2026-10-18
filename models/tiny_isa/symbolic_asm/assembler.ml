open Tiny_isa_shared.Instruction
open Types

let lower_inst raw =
  let m = raw.mnemonic and cf = raw.cond_flag in
  let arity n =
    if List.length raw.operands <> n then
      failwith (Printf.sprintf "%s requires %d operands" m n)
  in
  let no_flag () = if cf then failwith (m ^ " \\f is not supported") in
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
    let s = !symbols in
    if
      List.mem_assoc name s.consts
      || List.mem_assoc name s.text_labels
      || List.mem_assoc name s.data_labels
    then
      failwith
        ("Duplicate symbol definition: " ^ Utils_names.Identifier.to_string name)
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
            failwith "Instruction found outside \\text section";
          symbols := { !symbols with code_len = !symbols.code_len + 1 }
      | DataValue _ ->
          if !section <> Data then
            failwith "Data value found outside \\data section";
          symbols := { !symbols with data_len = !symbols.data_len + 1 })
    code;
  !symbols

let resolve_raw_value symbols section this_pos = function
  | Imm n -> n
  | Const n -> (
      match List.assoc_opt n symbols.consts with
      | Some n -> n
      | None ->
          failwith
            ("Unknown const reference: " ^ Utils_names.Identifier.to_string n))
  | Builtin This -> N.of_int this_pos
  | Builtin Code_len -> N.of_int symbols.code_len
  | Builtin Data_len -> N.of_int symbols.data_len
  | Label n -> (
      match List.assoc_opt n symbols.text_labels with
      | Some n -> N.of_int n
      | None -> (
          match List.assoc_opt n symbols.data_labels with
          | Some n -> N.of_int n
          | None ->
              failwith
                ("Unknown label reference in "
                ^ (if section = Text then "\\text" else "\\data")
                ^ ": @"
                ^ Utils_names.Identifier.to_string n)))

let build_debug_info symbols =
  let by_addr labels =
    List.fold_left
      (fun out (name, addr) ->
        let names = Option.value ~default:[] (List.assoc_opt addr out) in
        (addr, List.sort Utils_names.Identifier.compare (name :: names))
        :: List.remove_assoc addr out)
      [] labels
    |> List.sort (fun (a, _) (b, _) -> Int.compare a b)
  in
  {
    text_labels_by_addr = by_addr symbols.text_labels;
    data_labels_by_addr = by_addr symbols.data_labels;
  }

let preprocess code =
  Utils_base.Error.protect (fun () ->
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
                failwith "Instruction found outside \\text section";
              insts := lower_inst raw :: !insts
          | DataValue v ->
              if !section <> Data then
                failwith "Data value found outside \\data section";
              let n = resolve_raw_value symbols Data !pos v in
              incr pos;
              data := n :: !data)
        code;
      ( Array.of_list (List.rev !insts),
        List.rev !data,
        symbols,
        build_debug_info symbols ))
