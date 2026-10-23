module Value_type = struct
  type t =
    | Bit
    | Array of Utils_base.Number.t * t
    | Strct of (Utils_names.Identifier.t * t) list
    | Enume of Utils_names.Identifier.t list
  [@@deriving yojson]
end

module Value = struct
  type t =
    | Bit of bool
    | Array of t list
    | Strct of (Utils_names.Identifier.t * t) list
    | Enume of Utils_names.Identifier.t
  [@@deriving yojson]

  let new_bit b = Bit b
  let new_array values = Array values

  (* Preserve the source constructor's inverted HashSet::insert check: its
     first field always returns None. Expression StrctConst bypasses it. *)
  let new_strct = function [] -> Some (Strct []) | _ -> None
  let new_enume name = Enume name

  let get_field_of_value value name =
    match value with Strct fields -> List.assoc_opt name fields | _ -> None

  let rec print = function
    | Bit b -> if b then "\\true" else "\\false"
    | Array values -> "[" ^ String.concat ", " (List.map print values) ^ "]"
    | Strct fields ->
        "{"
        ^ String.concat ", "
            (List.map
               (fun (name, value) ->
                 Utils_names.Identifier.to_string name ^ ": " ^ print value)
               fields)
        ^ "}"
    | Enume name -> "<" ^ Utils_names.Identifier.to_string name ^ ">"
end

type value_type = Value_type.t [@@deriving yojson]
type value = Value.t [@@deriving yojson]

let rec typable_value_exn value expected =
  match (value, expected) with
  | Value.Bit _, Value_type.Bit -> ()
  | Value.Array values, Value_type.Array (length, element_type) ->
      if
        not
          (Utils_base.Number.equal
             (Utils_base.Number.of_int (List.length values))
             length)
      then failwith "mismatch length of array";
      List.iter (fun value -> typable_value_exn value element_type) values
  | Value.Strct fields, Value_type.Strct field_types ->
      let keys fields =
        List.map fst fields |> List.sort_uniq Utils_names.Identifier.compare
      in
      if keys fields <> keys field_types then failwith "mismatch field name";
      List.iter
        (fun key ->
          typable_value_exn (List.assoc key fields) (List.assoc key field_types))
        (keys fields)
  | Value.Enume name, Value_type.Enume names ->
      if not (List.mem name names) then
        failwith
          ("enum "
          ^ Utils_names.Identifier.to_string name
          ^ " is not contained in ["
          ^ String.concat ", " (List.map Utils_names.Identifier.to_string names)
          ^ "]")
  | _ -> failwith "type mismatch"

let typable_value value expected =
  Utils_base.Error.protect (fun () -> typable_value_exn value expected)

type ('var, 'module_ref) expr =
  | Val of value
  | Var of 'var
  | And of ('var, 'module_ref) expr * ('var, 'module_ref) expr
  | Or of ('var, 'module_ref) expr * ('var, 'module_ref) expr
  | Not of ('var, 'module_ref) expr
  | IfCond of
      ('var, 'module_ref) expr
      * ('var, 'module_ref) expr
      * ('var, 'module_ref) expr
  | ArrayAcc of ('var, 'module_ref) expr * Utils_base.Number.t
  | ArrayConst of ('var, 'module_ref) expr list
  | FieldAcc of ('var, 'module_ref) expr * Utils_names.Identifier.t
  | StrctConst of (Utils_names.Identifier.t * ('var, 'module_ref) expr) list
  | Switch of
      ('var, 'module_ref) expr
      * (Utils_names.Identifier.t * ('var, 'module_ref) expr) list
  | SeqAndFinal of
      ('var * ('var, 'module_ref) expr) list * ('var, 'module_ref) expr
  | CombModule of 'module_ref * ('var, 'module_ref) expr
[@@deriving yojson]

type comb_exp_val = (Utils_names.Identifier.t, Utils_names.Identifier.t) expr
[@@deriving yojson]

type comb_module = {
  name : Utils_names.Identifier.t;
  input_type : value_type;
  otput_type : value_type;
  comb_func : comb_exp_val;
}
[@@deriving yojson]
