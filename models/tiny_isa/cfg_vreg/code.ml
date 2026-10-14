open Syntax

type t = code

let parse_vreg s =
  if not (String.starts_with ~prefix:"%v" s) then failwith ("Invalid vreg: " ^ s)
  else
    Utils_base.Error.get
      (Utils_codec.Text.Natural_int.parse (Text.strip_prefix "%v" s))

module P = Utils_parsing.Parser

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

let parse_addr_expr text = Utils_base.Error.get (P.parse read_addr text)
let parse_place_expr text = Utils_base.Error.get (P.parse read_place text)
let parse_value_expr text = Utils_base.Error.get (P.parse read_value text)
let parse_cond text = Utils_base.Error.get (P.parse read_cond text)

let parse_stmt text =
  if String.ends_with ~suffix:";" text && Text.split_once ":=" text <> None then
    Some (Utils_base.Error.get (P.parse read_stmt text))
  else None

let parse_jump_if text =
  if String.starts_with ~prefix:"if " text && String.ends_with ~suffix:";" text
  then Some (Utils_base.Error.get (P.parse read_jump_if text))
  else None

let parse_goto text =
  if
    String.starts_with ~prefix:"goto " text && String.ends_with ~suffix:";" text
  then Some (Utils_base.Error.get (P.parse read_goto text))
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
                  | None -> P.error p ("Block @" ^ label ^ " missing final goto")
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
