open Syntax

type t = code

let parse_reg token =
  let t = Text.strip_prefix "r" (Text.strip_prefix "%" (String.trim token)) in
  let n = Utils_base.Error.get (Utils_data.Text.Natural_int.parse t) in
  if n >= 8 then
    failwith (Printf.sprintf "Register out of range (expected 0..=7): %d" n);
  n

let parse_raw_value token =
  let t = String.trim token in
  if t = "" then failwith "Empty value";
  if String.starts_with ~prefix:"#" t then
    Imm (Utils_base.Error.get (N.of_string_with_plus (Text.strip_prefix "#" t)))
  else if String.starts_with ~prefix:"@" t then
    let name = Text.strip_prefix "@" t in
    Label (Utils_base.Error.get (Utils_names.Identifier.make name))
  else if String.for_all (function '0' .. '9' -> true | _ -> false) t then
    Imm (Utils_base.Error.get (N.of_string_with_plus t))
  else Const (Utils_base.Error.get (Utils_names.Identifier.make t))

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
        ( Utils_base.Error.get (Utils_names.Identifier.make name),
          Utils_base.Error.get (N.of_string_with_plus (Text.strip_prefix "#" n))
        )
  | _ -> failwith ("Unsupported directive syntax: " ^ line)

let parse text =
  Utils_parsing.Parser.protect (fun () ->
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
                LabelDef (Utils_base.Error.get (Utils_names.Identifier.make n))
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
  | Label n -> "@" ^ Utils_names.Identifier.to_string n
  | Const n -> Utils_names.Identifier.to_string n

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
  | Directive (Equ (n, v)) ->
      ".equ " ^ Utils_names.Identifier.to_string n ^ " #" ^ N.to_string v
  | LabelDef n -> "@" ^ Utils_names.Identifier.to_string n ^ ":"
  | Inst i -> raw_inst_to_text i
  | DataValue v -> raw_value_to_text v

let print code = String.concat "\n" (List.map item_to_text code)
