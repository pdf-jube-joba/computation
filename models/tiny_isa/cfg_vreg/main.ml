module Text = Utils.String_ext
open Tiny_isa_shared.Instruction

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

module Code = struct
  type t = code

  let parse_vreg s =
    if not (String.starts_with ~prefix:"%v" s) then
      failwith ("Invalid vreg: " ^ s)
    else value (Utils.Text.Natural_int.parse (Text.strip_prefix "%v" s))

  module P = Utils.Parser

  let read_vreg p =
    P.expect_symbol p '%';
    parse_vreg ("%" ^ P.identifier p)

  let read_addr p =
    if P.peek p = P.Symbol '%' then VReg (read_vreg p)
    else if P.bump_if_symbol p '@' then Label (P.identifier p)
    else if P.bump_if_symbol p '#' then Imm (P.natural_with_plus p)
    else Imm (P.natural p)

  let read_place p = P.group p ~opening:'[' read_addr

  let read_value p =
    if P.bump_if_symbol p '*' then Deref (read_place p)
    else
      match read_addr p with
      | VReg n -> VRegValue n
      | Label n -> LabelValue n
      | Imm n -> ImmValue n

  let read_stmt p =
    let stmt =
      if P.peek p = P.Symbol '%' then (
        let dst = read_vreg p in
        P.expect p ":=";
        let lhs = read_value p in
        if P.bump_if_symbol p '+' then BinOp (dst, lhs, Add, read_value p)
        else if P.bump_if_symbol p '-' then BinOp (dst, lhs, Sub, read_value p)
        else Assign (dst, lhs))
      else
        let dst = read_place p in
        P.expect p ":=";
        Store (dst, read_value p)
    in
    P.expect_symbol p ';';
    stmt

  let read_cond p =
    P.binary p
      ~operators:
        [
          ("<", fun a b -> Lt (a, b));
          ("=", fun a b -> Eq (a, b));
          (">", fun a b -> Gt (a, b));
        ]
      ~operand:read_value
      ~make:(fun a op b -> op a b)

  let read_jump_if p =
    P.expect_keyword p "if";
    let cond = read_cond p in
    P.expect_keyword p "then";
    let target = read_addr p in
    P.expect_symbol p ';';
    { cond; target }

  let read_goto p =
    P.expect_keyword p "goto";
    let target = read_addr p in
    P.expect_symbol p ';';
    target

  let parse_addr_expr text = value (P.parse read_addr text)
  let parse_place_expr text = value (P.parse read_place text)
  let parse_value_expr text = value (P.parse read_value text)
  let parse_cond text = value (P.parse read_cond text)

  let parse_stmt text =
    if String.ends_with ~suffix:";" text && Text.split_once ":=" text <> None
    then Some (value (P.parse read_stmt text))
    else None

  let parse_jump_if text =
    if
      String.starts_with ~prefix:"if " text && String.ends_with ~suffix:";" text
    then Some (value (P.parse read_jump_if text))
    else None

  let parse_goto text =
    if
      String.starts_with ~prefix:"goto " text
      && String.ends_with ~suffix:";" text
    then Some (value (P.parse read_goto text))
    else None

  let parse =
    P.parse (fun p ->
        let block p label =
          P.group p ~opening:'{' (fun p ->
              let rec body stmts ifs jump =
                if P.eof p then
                  let jump =
                    match jump with
                    | Some target -> target
                    | None ->
                        P.error p ("Block @" ^ label ^ " missing final goto")
                  in
                  {
                    label;
                    stmts = List.rev stmts;
                    cont = { ifs = List.rev ifs; jump };
                  }
                else if P.peek p = P.Ident "if" then
                  let jump_if = read_jump_if p in
                  body stmts (jump_if :: ifs) jump
                else if P.peek p = P.Ident "goto" then (
                  if jump <> None then
                    P.error p ("Multiple final goto in block @" ^ label);
                  let target = read_goto p in
                  body stmts ifs (Some target))
                else (
                  if jump <> None then
                    P.error p ("Statement after goto in block @" ^ label);
                  let stmt = read_stmt p in
                  body (stmt :: stmts) ifs jump)
              in
              body [] [] None)
        in
        let rec top statics blocks =
          if P.eof p then
            ({ statics = List.rev statics; blocks = List.rev blocks } : code)
          else (
            P.expect_symbol p '@';
            let label = P.identifier p in
            if P.peek p = P.Symbol '{' then
              let b = block p label in
              top statics (b :: blocks)
            else
              let value = P.natural_with_plus p in
              top (({ label; value } : static_def) :: statics) blocks)
        in
        top [] [])

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
    | Store (place, src) ->
        place_to_text place ^ " := " ^ value_to_text src ^ ";"

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
end

let compile_program (code : code) =
  Utils.Parser.protect (fun () ->
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
  Utils.Parser.protect (fun () ->
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
  Utils.Parser.protect (fun () ->
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
        Code.stmt_to_text
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
