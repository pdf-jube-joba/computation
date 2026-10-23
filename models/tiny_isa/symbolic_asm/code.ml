open Syntax

type t = code

module P = Utils_parsing.Parser

let parse_reg token =
  let t = Text.strip_prefix "r" (Text.strip_prefix "%" (String.trim token)) in
  let n = Utils_base.Error.get (Utils_data.Text.Natural_int.parse t) in
  if n >= 8 then
    failwith (Printf.sprintf "Register out of range (expected 0..=7): %d" n);
  n

let read_raw_value p =
  if P.bump_if_symbol p '#' then Imm (P.natural_with_plus p)
  else if P.bump_if_symbol p '@' then Label (P.identifier p)
  else
    match P.peek p with
    | P.Number _ -> Imm (P.natural p)
    | P.Keyword _ ->
        Builtin
          (match P.keyword p with
          | "THIS" -> This
          | "CODE_LEN" -> Code_len
          | "DATA_LEN" -> Data_len
          | _ -> P.error p "Expected a builtin address")
    | _ -> Const (P.identifier p)

let parse_raw_value text = Utils_base.Error.get (P.parse read_raw_value text)

let parse_inst_line line =
  Utils_base.Error.get
    (P.parse
       (fun p ->
         let mnemonic = P.keyword p in
         let cond_flag = P.bump_if_keyword p "f" in
         let operands =
           P.many p (fun p ->
               if P.bump_if_symbol p '%' then
                 Reg
                   (parse_reg
                      (Utils_names.Identifier.to_string (P.identifier p)))
               else Value (read_raw_value p))
         in
         { mnemonic; cond_flag; operands })
       (String.trim (Text.strip_suffix ";" (String.trim line))))

let parse_directive line =
  Utils_base.Error.get
    (P.parse
       (fun p ->
         match P.keyword p with
         | "text" -> Text
         | "data" -> Data
         | "equ" ->
             let name = P.identifier p in
             ignore (P.bump_if_symbol p '#');
             Equ (name, P.natural_with_plus p)
         | _ -> P.error p "Expected a section or constant directive")
       line)

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
              else if
                match P.peek (P.create line) with
                | P.Keyword ("text" | "data" | "equ") -> true
                | _ -> false
              then (
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
  | Builtin This -> "\\THIS"
  | Builtin Code_len -> "\\CODE_LEN"
  | Builtin Data_len -> "\\DATA_LEN"

let operand_to_text = function
  | Reg r -> "%r" ^ string_of_int r
  | Value v -> raw_value_to_text v

let raw_inst_to_text i =
  "\\" ^ i.mnemonic
  ^ (if i.cond_flag then " \\f" else "")
  ^ String.concat "" (List.map (fun op -> " " ^ operand_to_text op) i.operands)
  ^ ";"

let item_to_text = function
  | Directive Text -> "\\text"
  | Directive Data -> "\\data"
  | Directive (Equ (n, v)) ->
      "\\equ " ^ Utils_names.Identifier.to_string n ^ " #" ^ N.to_string v
  | LabelDef n -> "@" ^ Utils_names.Identifier.to_string n ^ ":"
  | Inst i -> raw_inst_to_text i
  | DataValue v -> raw_value_to_text v

let print code = String.concat "\n" (List.map item_to_text code)
