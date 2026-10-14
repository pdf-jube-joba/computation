open Runtime_types

let location_text loc =
  (match loc.key with
    | Heap n -> "heap#" ^ string_of_int (Heap_id.to_int n)
    | Static_key n -> "static " ^ Utils_names.Identifier.to_string n
    | Local_key n -> "local#" ^ string_of_int (Local_id.to_int n))
  ^ String.concat ""
      (List.map
         (function
           | Field_proj n -> "." ^ string_of_int n
           | Tag_proj n -> "?" ^ string_of_int n
           | Index_proj n -> "[" ^ string_of_int n ^ "]")
         loc.projections)

let rec value_text = function
  | Number_value n -> N.to_string n
  | Char_value c -> "'" ^ c ^ "'"
  | Bool_value b -> if b then "#true" else "#false"
  | Unit_v -> "#unit"
  | Ptr_value None -> "#null_ptr"
  | Ptr_value (Some l) -> location_text l
  | FnPtr None -> "#null_fn"
  | FnPtr (Some n) -> "fn " ^ Utils_names.Identifier.to_string n
  | Aggregate vs -> "(" ^ String.concat ", " (List.map value_text vs) ^ ")"
  | Sum_value (t, v) -> "tag(" ^ string_of_int t ^ ", " ^ value_text v ^ ")"

let exec_text = function
  | Eval s -> "eval " ^ Print.stmt_text s
  | Ctrl Next -> "ctrl next"
  | Ctrl (Break_ctrl l) -> "ctrl break " ^ Utils_names.Identifier.to_string l
  | Ctrl (Continue_ctrl l) ->
      "ctrl continue " ^ Utils_names.Identifier.to_string l
  | Ret vs -> "ret " ^ String.concat ", " (List.map value_text vs)

let inner_text = function
  | Stmt_frame s -> "stmt " ^ Print.stmt_text s
  | Scope -> "scope"
  | Loop_frame (l, s) ->
      "loop " ^ Utils_names.Identifier.to_string l ^ ": " ^ Print.stmt_text s
