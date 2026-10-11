module Text = Utils.String_ext
open Isa_common

type addr_expr = VReg of int | Label of Utils.Identifier.t | Imm of N.t
[@@deriving yojson]

type place_expr = addr_expr [@@deriving yojson]

type value_expr =
  | VRegValue of int
  | ImmValue of N.t
  | LabelValue of Utils.Identifier.t
  | Deref of place_expr
[@@deriving yojson]

type binop = Add | Sub [@@deriving yojson]

type cond =
  | Lt of value_expr * value_expr
  | Eq of value_expr * value_expr
  | Gt of value_expr * value_expr
[@@deriving yojson]

type stmt =
  | Assign of int * value_expr
  | BinOp of int * value_expr * binop * value_expr
  | Store of place_expr * value_expr
[@@deriving yojson]

type jump_if = { cond : cond; target : addr_expr } [@@deriving yojson]
type cont = { ifs : jump_if list; jump : addr_expr } [@@deriving yojson]

type block = { label : Utils.Identifier.t; stmts : stmt list; cont : cont }
[@@deriving yojson]

type static_def = { label : Utils.Identifier.t; value : N.t }
[@@deriving yojson]

type code = { statics : static_def list; blocks : block list }
[@@deriving yojson]

type compiled_program = {
  blocks : block array;
  entry_block : int;
  block_index : (Utils.Identifier.t * int) list;
  static_labels : (Utils.Identifier.t * int) list;
}
[@@deriving yojson]

type ainput = N.t list
type rinput = unit
type routput = unit
type foutput = N.t list

type state = {
  code : code;
  compiled : compiled_program;
  current_block : int;
  vregs : N.t array;
  memory : N.t array;
  halted : bool;
}
[@@deriving yojson]

let parse_vreg s =
  if not (String.starts_with ~prefix:"%v" s) then failwith ("Invalid vreg: " ^ s)
  else value (Utils.Text.Natural_int.parse (Text.strip_prefix "%v" s))

let parse_addr_expr s =
  let s = String.trim s in
  if String.starts_with ~prefix:"%v" s then VReg (parse_vreg s)
  else if String.starts_with ~prefix:"@" s then
    Label (value (Utils.Identifier.make (Text.strip_prefix "@" s)))
  else if String.starts_with ~prefix:"#" s then
    Imm (value (number (Text.strip_prefix "#" s)))
  else if
    s <> "" && String.for_all (function '0' .. '9' -> true | _ -> false) s
  then Imm (value (number s))
  else failwith ("Invalid addr expr: " ^ s)

let parse_place_expr s =
  let s = String.trim s in
  if not (String.starts_with ~prefix:"[" s && String.ends_with ~suffix:"]" s)
  then failwith ("Invalid place expr: " ^ s);
  parse_addr_expr
    (String.trim (Text.strip_suffix "]" (Text.strip_prefix "[" s)))

let parse_value_expr s =
  let s = String.trim s in
  if String.starts_with ~prefix:"*" s then
    Deref (parse_place_expr (Text.strip_prefix "*" s))
  else
    match parse_addr_expr s with
    | VReg n -> VRegValue n
    | Label n -> LabelValue n
    | Imm n -> ImmValue n

let parse_stmt line =
  if not (String.ends_with ~suffix:";" line) then None
  else
    match Text.split_once ":=" (String.trim (Text.strip_suffix ";" line)) with
    | None -> None
    | Some (lhs, rhs) ->
        let lhs = String.trim lhs and rhs = String.trim rhs in
        if String.starts_with ~prefix:"%v" lhs then
          let dst = parse_vreg lhs in
          match Text.words rhs with
          | [ src ] -> Some (Assign (dst, parse_value_expr src))
          | [ lhs; op; rhs ] ->
              let lhs = parse_value_expr lhs and rhs = parse_value_expr rhs in
              let op =
                match op with
                | "+" -> Add
                | "-" -> Sub
                | _ -> failwith ("Unsupported op: " ^ op)
              in
              Some (BinOp (dst, lhs, op, rhs))
          | _ -> failwith ("Invalid assignment stmt: " ^ line)
        else if String.starts_with ~prefix:"[" lhs then
          Some (Store (parse_place_expr lhs, parse_value_expr rhs))
        else None

let parse_cond s =
  let make sep ctor =
    Option.map
      (fun (a, b) ->
        ctor (parse_value_expr (String.trim a), parse_value_expr (String.trim b)))
      (Text.split_once sep s)
  in
  match make " < " (fun (a, b) -> Lt (a, b)) with
  | Some c -> c
  | None -> (
      match make " = " (fun (a, b) -> Eq (a, b)) with
      | Some c -> c
      | None -> (
          match make " > " (fun (a, b) -> Gt (a, b)) with
          | Some c -> c
          | None -> failwith ("Invalid condition: " ^ s)))

let parse_jump_if line =
  if
    not
      (String.starts_with ~prefix:"if " line
      && String.ends_with ~suffix:";" line)
  then None
  else
    match
      Text.split_once " then "
        (Text.strip_prefix "if " (Text.strip_suffix ";" line))
    with
    | Some (c, t) ->
        Some
          {
            cond = parse_cond (String.trim c);
            target = parse_addr_expr (String.trim t);
          }
    | None -> failwith ("Invalid if syntax: " ^ line)

let parse_goto line =
  if
    String.starts_with ~prefix:"goto " line && String.ends_with ~suffix:";" line
  then
    Some
      (parse_addr_expr (Text.strip_prefix "goto " (Text.strip_suffix ";" line)))
  else None

let parse_code text =
  Utils.Lexer.protect (fun () ->
      let lines =
        Array.of_list
          (List.filter (( <> ) "")
             (List.map
                (fun l -> String.trim (Text.strip_line_comment "//" l))
                (String.split_on_char '\n' text)))
      in
      let rec top i statics blocks =
        if i = Array.length lines then
          ({ statics = List.rev statics; blocks = List.rev blocks } : code)
        else
          let line = lines.(i) in
          if
            String.starts_with ~prefix:"@" line
            && String.ends_with ~suffix:"{" line
          then
            let label =
              String.trim (Text.strip_suffix "{" (Text.strip_prefix "@" line))
            in
            let label = value (Utils.Identifier.make label) in
            let rec block j stmts ifs jump =
              if j = Array.length lines then
                failwith ("Unclosed block: @" ^ label);
              let line = lines.(j) in
              if line = "}" then
                let jump =
                  match jump with
                  | Some t -> t
                  | None -> failwith ("Block @" ^ label ^ " missing final goto")
                in
                ( {
                    label;
                    stmts = List.rev stmts;
                    cont = { ifs = List.rev ifs; jump };
                  },
                  j + 1 )
              else
                match parse_jump_if line with
                | Some jump_if -> block (j + 1) stmts (jump_if :: ifs) jump
                | None -> (
                    match parse_goto line with
                    | Some target ->
                        if jump <> None then
                          failwith ("Multiple final goto in block @" ^ label);
                        block (j + 1) stmts ifs (Some target)
                    | None -> (
                        if jump <> None then
                          failwith ("Statement after goto in block @" ^ label);
                        match parse_stmt line with
                        | Some stmt -> block (j + 1) (stmt :: stmts) ifs jump
                        | None ->
                            failwith
                              ("Invalid line in block @" ^ label ^ ": " ^ line))
                    )
            in
            let b, j = block (i + 1) [] [] None in
            top j statics (b :: blocks)
          else
            match Text.words line with
            | [ label; n ]
              when String.starts_with ~prefix:"@" label
                   && not (String.ends_with ~suffix:":" label) ->
                let s : static_def =
                  {
                    label =
                      value
                        (Utils.Identifier.make (Text.strip_prefix "@" label));
                    value = value (number n);
                  }
                in
                top (i + 1) (s :: statics) blocks
            | _ -> failwith ("Unexpected top-level line: " ^ line)
      in
      top 0 [] [])

let addr_to_text = function
  | VReg i -> "%v" ^ string_of_int i
  | Imm n -> "#" ^ N.to_string n
  | Label n -> "@" ^ n

let place_to_text p = "[" ^ addr_to_text p ^ "]"

let value_to_text = function
  | VRegValue i -> "%v" ^ string_of_int i
  | ImmValue n -> "#" ^ N.to_string n
  | LabelValue n -> "@" ^ n
  | Deref p -> "*" ^ place_to_text p

let cond_to_text = function
  | Lt (a, b) -> value_to_text a ^ " < " ^ value_to_text b
  | Eq (a, b) -> value_to_text a ^ " = " ^ value_to_text b
  | Gt (a, b) -> value_to_text a ^ " > " ^ value_to_text b

let stmt_to_text = function
  | Assign (dst, src) -> Printf.sprintf "%%v%d := %s;" dst (value_to_text src)
  | BinOp (dst, lhs, op, rhs) ->
      Printf.sprintf "%%v%d := %s %s %s;" dst (value_to_text lhs)
        (match op with Add -> "+" | Sub -> "-")
        (value_to_text rhs)
  | Store (place, src) -> place_to_text place ^ " := " ^ value_to_text src ^ ";"

let print (code : code) =
  String.concat ""
    (List.map
       (fun (s : static_def) ->
         "@" ^ s.label ^ " " ^ N.to_string s.value ^ "\n")
       code.statics
    @ List.map
        (fun (b : block) ->
          "@" ^ b.label ^ " {\n"
          ^ String.concat ""
              (List.map (fun s -> "  " ^ stmt_to_text s ^ "\n") b.stmts)
          ^ String.concat ""
              (List.map
                 (fun j ->
                   "  if " ^ cond_to_text j.cond ^ " then "
                   ^ addr_to_text j.target ^ ";\n")
                 b.cont.ifs)
          ^ "  goto " ^ addr_to_text b.cont.jump ^ ";\n}\n")
        code.blocks)

let compile_program (code : code) =
  Utils.Lexer.protect (fun () ->
      let block_index =
        List.mapi (fun i (b : block) -> (b.label, i)) code.blocks
      in
      let static_labels =
        List.mapi (fun i (s : static_def) -> (s.label, i)) code.statics
      in
      let unique kind labels =
        Utils.Identifier.check_unique
          ~message:(fun label -> "Duplicate " ^ kind ^ " label: @" ^ label)
          (List.map fst labels)
      in
      unique "block" block_index;
      unique "static" static_labels;
      let entry_block =
        match List.assoc_opt "main" block_index with
        | Some i -> i
        | None -> failwith "Missing entry block: @main"
      in
      {
        blocks = Array.of_list code.blocks;
        entry_block;
        block_index;
        static_labels;
      })

let make (code : code) ainput =
  Utils.Lexer.protect (fun () ->
      let compiled = value (compile_program code) in
      {
        code;
        compiled;
        current_block = compiled.entry_block;
        vregs = [||];
        memory =
          Array.of_list
            (List.map (fun (s : static_def) -> s.value) code.statics @ ainput);
        halted = false;
      })

let eval_addr s = function
  | VReg i -> read s.vregs i
  | Imm n -> n
  | Label n -> (
      match List.assoc_opt n s.compiled.static_labels with
      | Some i -> N.of_int i
      | None -> (
          match List.assoc_opt n s.compiled.block_index with
          | Some i -> N.of_int i
          | None -> failwith ("Unknown label: @" ^ n)))

let eval_place s p = value (N.to_int (eval_addr s p))

let eval_value s = function
  | VRegValue i -> read s.vregs i
  | ImmValue n -> n
  | LabelValue n -> eval_addr s (Label n)
  | Deref p -> read s.memory (eval_place s p)

let eval_cond s = function
  | Lt (a, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      N.compare a b < 0
  | Eq (a, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      N.equal a b
  | Gt (a, b) ->
      let a = eval_value s a in
      let b = eval_value s b in
      N.compare a b > 0

let step s () =
  Utils.Lexer.protect (fun () ->
      if s.halted then Utils.Machine.Halt (Array.to_list s.memory)
      else (
        if
          s.current_block < 0
          || s.current_block >= Array.length s.compiled.blocks
        then
          failwith
            (Printf.sprintf "Current block out of range: %d" s.current_block);
        let block = s.compiled.blocks.(s.current_block) in
        let s =
          List.fold_left
            (fun s -> function
              | Assign (dst, src) ->
                  let n = eval_value s src in
                  { s with vregs = write s.vregs dst n }
              | BinOp (dst, a, op, b) ->
                  let a = eval_value s a in
                  let b = eval_value s b in
                  {
                    s with
                    vregs =
                      write s.vregs dst
                        ((match op with Add -> N.add | Sub -> N.sub) a b);
                  }
              | Store (place, src) ->
                  let addr = eval_place s place in
                  let n = eval_value s src in
                  { s with memory = write s.memory addr n })
            s block.stmts
        in
        let target =
          match List.find_opt (fun j -> eval_cond s j.cond) block.cont.ifs with
          | Some j -> j.target
          | None -> block.cont.jump
        in
        let current_block = value (N.to_int (eval_addr s target)) in
        if current_block < 0 || current_block >= Array.length s.compiled.blocks
        then
          failwith
            (Printf.sprintf "Block address out of range: %d" current_block);
        Utils.Machine.Continue { next = { s with current_block }; output = () }))

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  let current_block =
    if s.current_block >= 0 && s.current_block < Array.length s.compiled.blocks
    then Some s.compiled.blocks.(s.current_block)
    else None
  in
  Utils.Render.state
    [
      Utils.Render.text_table ~title:"status" ~columns:[ "field"; "value" ]
        [
          [ "block"; string_of_int s.current_block ];
          [
            "label";
            (match current_block with Some b -> b.label | None -> "<invalid>");
          ];
          [ "halted"; string_of_bool s.halted ];
        ];
      Utils.Render.indexed_table ~title:"block" ~columns:[ "index"; "stmt" ]
        stmt_to_text
        (match current_block with None -> [] | Some b -> b.stmts);
      Utils.Render.indexed_table ~title:"vregs" ~columns:[ "vreg"; "value" ]
        ~index_text:(fun i -> "v" ^ string_of_int i)
        N.to_string (Array.to_list s.vregs);
      Utils.Render.indexed_table ~title:"memory" ~columns:[ "addr"; "value" ]
        N.to_string (Array.to_list s.memory);
    ]

module Ainput = Utils.Text.Natural_list
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Utils.Text.Natural_list

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
