open Syntax

let rec value_to_text = function
  | VReg r -> "%" ^ Utils_names.Identifier.to_string r
  | Imm n -> "#" ^ N.to_string n
  | CodeLabel l -> ":" ^ Utils_names.Identifier.to_string l
  | Ref p -> "ref " ^ place_to_text p

and place_to_text = function
  | Label l -> "@" ^ Utils_names.Identifier.to_string l
  | Deref r -> "deref %" ^ Utils_names.Identifier.to_string r
  | SAcc v -> "sacc " ^ value_to_text v
  | HAcc (h, i) -> "hacc(" ^ value_to_text h ^ ")[" ^ value_to_text i ^ "]"

let cond_to_text c =
  let a, op, b =
    match c with
    | Lt (a, b) -> (a, "<", b)
    | Eq (a, b) -> (a, "=", b)
    | Gt (a, b) -> (a, ">", b)
  in
  value_to_text a ^ " " ^ op ^ " " ^ value_to_text b

let stmt_to_text = function
  | Nop -> "nop;"
  | Load (r, p) ->
      "%"
      ^ Utils_names.Identifier.to_string r
      ^ " := ld " ^ place_to_text p ^ ";"
  | Assign (r, v) ->
      "%" ^ Utils_names.Identifier.to_string r ^ " := " ^ value_to_text v ^ ";"
  | BinOp (r, a, op, b) ->
      "%"
      ^ Utils_names.Identifier.to_string r
      ^ " := " ^ value_to_text a
      ^ (if op = Add then " + " else " - ")
      ^ value_to_text b ^ ";"
  | Store (p, v) -> place_to_text p ^ " := st " ^ value_to_text v ^ ";"
  | Pop r -> "pop %" ^ Utils_names.Identifier.to_string r ^ ";"
  | Push v -> "push " ^ value_to_text v ^ ";"
  | LGet r -> "lget %" ^ Utils_names.Identifier.to_string r ^ ";"
  | HAlloc (v, r) ->
      "halloc(" ^ value_to_text v ^ ") %"
      ^ Utils_names.Identifier.to_string r
      ^ ";"
  | HFree v -> "hfree " ^ value_to_text v ^ ";"
  | Print r -> "print %" ^ Utils_names.Identifier.to_string r ^ ";"
  | Input p -> "input " ^ place_to_text p ^ ";"

let print (p : code) =
  let b = Buffer.create 512 in
  let line s = Buffer.add_string b (s ^ "\n") in
  List.iter
    (fun (s : static_def) ->
      line
        ("@"
        ^ Utils_names.Identifier.to_string s.label
        ^ " " ^ N.to_string s.value))
    p.statics;
  List.iter
    (fun (r : region) ->
      line (":" ^ Utils_names.Identifier.to_string r.label ^ " {");
      List.iter
        (fun (bb : block) ->
          line ("  :" ^ Utils_names.Identifier.to_string bb.label ^ " {");
          List.iter (fun s -> line ("    " ^ stmt_to_text s)) bb.stmts;
          List.iter
            (fun j ->
              line
                ("    if " ^ cond_to_text j.cond ^ " then "
               ^ value_to_text j.target ^ ";"))
            (ifs bb.cont);
          line
            ("    "
            ^
            match bb.cont with
            | Go (_, v) -> "goto " ^ value_to_text v ^ ";"
            | Enter (_, v) -> "enter " ^ value_to_text v ^ ";"
            | Halt _ -> "halt;");
          line "  }")
        r.blocks;
      line "}")
    p.regions;
  Buffer.contents b
