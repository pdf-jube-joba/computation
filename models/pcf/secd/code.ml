open Pcf_shared.Syntax
open Types

let ( let* ) = Result.bind

(* Public code retains Rust serde's externally tagged instruction syntax. *)
type t = code

let json_nat n =
  match N.to_int n with Ok i -> `Int i | Error _ -> `Intlit (N.to_string n)

let rec instr_json (i : instr) : Yojson.Safe.t =
  match i with
  | PushNat n -> `Assoc [ ("PushNat", json_nat n) ]
  | PushBool b -> `Assoc [ ("PushBool", `Bool b) ]
  | PushUnit -> `String "PushUnit"
  | Print n -> `Assoc [ ("Print", json_nat n) ]
  | Acc x -> `Assoc [ ("Acc", `String (Utils_names.Identifier.to_string x)) ]
  | MkCls (x, c) ->
      `Assoc
        [
          ( "MkCls",
            `Assoc
              [
                ("param", `String (Utils_names.Identifier.to_string x));
                ("code", code_json c);
              ] );
        ]
  | MkRec (f, x, c) ->
      `Assoc
        [
          ( "MkRec",
            `Assoc
              [
                ("name", `String (Utils_names.Identifier.to_string f));
                ("param", `String (Utils_names.Identifier.to_string x));
                ("code", code_json c);
              ] );
        ]
  | BinOp o ->
      `Assoc
        [
          ( "BinOp",
            `String (match o with Add -> "Add" | Sub -> "Sub" | And -> "And") );
        ]
  | UnOp o ->
      `Assoc
        [
          ( "UnOp",
            `String (match o with Inc -> "Inc" | Dec -> "Dec" | Not -> "Not") );
        ]
  | Ap -> `String "Ap"
  | Ret -> `String "Ret"
  | If (t, e) ->
      `Assoc
        [
          ( "If",
            `Assoc [ ("then_code", code_json t); ("else_code", code_json e) ] );
        ]

and code_json (c : code) : Yojson.Safe.t = `List (List.map instr_json c)

let get = Utils_base.Error.get

let natural_json = function
  | `Int n when n >= 0 -> N.of_int n
  | `Intlit s -> get (N.of_string s)
  | _ -> failwith "expected natural number"

let identifier_json j =
  get (Utils_names.Identifier.make (Utils_data.Json.string j))

let rec instr_from_json = function
  | `String "PushUnit" -> PushUnit
  | `String "Ap" -> Ap
  | `String "Ret" -> Ret
  | `Assoc [ ("PushNat", j) ] -> PushNat (natural_json j)
  | `Assoc [ ("PushBool", `Bool b) ] -> PushBool b
  | `Assoc [ ("Print", j) ] -> Print (natural_json j)
  | `Assoc [ ("Acc", j) ] -> Acc (identifier_json j)
  | `Assoc [ ("MkCls", j) ] ->
      MkCls
        ( identifier_json (Utils_data.Json.field "param" j),
          code_from_json (Utils_data.Json.field "code" j) )
  | `Assoc [ ("MkRec", j) ] ->
      MkRec
        ( identifier_json (Utils_data.Json.field "name" j),
          identifier_json (Utils_data.Json.field "param" j),
          code_from_json (Utils_data.Json.field "code" j) )
  | `Assoc [ ("BinOp", `String o) ] ->
      BinOp
        (match o with
        | "Add" -> Add
        | "Sub" -> Sub
        | "And" -> And
        | _ -> failwith "unknown binop")
  | `Assoc [ ("UnOp", `String o) ] ->
      UnOp
        (match o with
        | "Inc" -> Inc
        | "Dec" -> Dec
        | "Not" -> Not
        | _ -> failwith "unknown unop")
  | `Assoc [ ("If", j) ] ->
      If
        ( code_from_json (Utils_data.Json.field "then_code" j),
          code_from_json (Utils_data.Json.field "else_code" j) )
  | _ -> failwith "invalid SECD instruction"

and code_from_json j = List.map instr_from_json (Utils_data.Json.list j)

let parse text =
  let* j = Utils_data.Json.decode text in
  Utils_data.Json.protect (fun () -> code_from_json j)

let print c = Yojson.Safe.pretty_to_string (code_json c)
