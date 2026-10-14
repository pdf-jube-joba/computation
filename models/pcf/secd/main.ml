open Pcf_shared.Syntax

let ( let* ) = Result.bind

include Types

(* Public code retains Rust serde's externally tagged instruction syntax. *)
module Code = struct
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
              `String
                (match o with Add -> "Add" | Sub -> "Sub" | And -> "And") );
          ]
    | UnOp o ->
        `Assoc
          [
            ( "UnOp",
              `String
                (match o with Inc -> "Inc" | Dec -> "Dec" | Not -> "Not") );
          ]
    | Ap -> `String "Ap"
    | Ret -> `String "Ret"
    | If (t, e) ->
        `Assoc
          [
            ( "If",
              `Assoc [ ("then_code", code_json t); ("else_code", code_json e) ]
            );
          ]

  and code_json (c : code) : Yojson.Safe.t = `List (List.map instr_json c)

  let get = Utils_base.Error.get

  let natural_json = function
    | `Int n when n >= 0 -> N.of_int n
    | `Intlit s -> get (N.of_string s)
    | _ -> failwith "expected natural number"

  let identifier_json j =
    get (Utils_names.Identifier.make (Utils_codec.Json.string j))

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
          ( identifier_json (Utils_codec.Json.field "param" j),
            code_from_json (Utils_codec.Json.field "code" j) )
    | `Assoc [ ("MkRec", j) ] ->
        MkRec
          ( identifier_json (Utils_codec.Json.field "name" j),
            identifier_json (Utils_codec.Json.field "param" j),
            code_from_json (Utils_codec.Json.field "code" j) )
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
          ( code_from_json (Utils_codec.Json.field "then_code" j),
            code_from_json (Utils_codec.Json.field "else_code" j) )
    | _ -> failwith "invalid SECD instruction"

  and code_from_json j = List.map instr_from_json (Utils_codec.Json.list j)

  let parse text =
    let* j = Utils_codec.Json.decode text in
    Utils_codec.Json.protect (fun () -> code_from_json j)

  let print c = Yojson.Safe.pretty_to_string (code_json c)
end

let make code inputs =
  Ok
    {
      code;
      stack = [];
      env = [];
      control = code @ List.concat_map (fun n -> [ PushNat n; Ap ]) inputs;
      dump = [];
    }

include Pcf_shared.Primitive.Make (struct
  type t = value

  let view = function
    | VNat n -> Pcf_shared.Primitive.Nat n
    | VBool b -> Pcf_shared.Primitive.Bool b
    | _ -> Pcf_shared.Primitive.Other

  let natural n = VNat n
  let boolean b = VBool b
end)

let as_nat = function
  | VNat n -> Ok n
  | _ -> Error "final value is not a natural number"

let value_text = function
  | VNat n -> N.to_string n
  | VBool b -> if b then "#true" else "#false"
  | VUnit -> "#unit"
  | VFun (x, c, _) ->
      "closure " ^ Utils_names.Identifier.to_string x ^ " " ^ Code.print c
  | VRec (f, x, c, _) ->
      "recursive closure "
      ^ Utils_names.Identifier.to_string f
      ^ " "
      ^ Utils_names.Identifier.to_string x
      ^ " " ^ Code.print c

let lookup_env (env : env) x =
  match List.assoc_opt x env with
  | Some v -> Ok v
  | None -> Error ("unbound variable: " ^ Utils_names.Identifier.to_string x)

let step_once s =
  match s.control with
  | [] -> Error "no instruction to execute"
  | instr :: control -> (
      let s = { s with control } in
      let unchanged s = Ok (s, None) in
      let push v = unchanged { s with stack = v :: s.stack } in
      match instr with
      | PushNat n -> push (VNat n)
      | PushBool b -> push (VBool b)
      | PushUnit -> push VUnit
      | Print n -> Ok ({ s with stack = VUnit :: s.stack }, Some n)
      | Acc x ->
          let* v = lookup_env s.env x in
          push v
      | MkCls (x, c) -> push (VFun (x, c, s.env))
      | MkRec (f, x, c) -> push (VRec (f, x, c, s.env))
      | BinOp o -> (
          match s.stack with
          | b :: a :: stack ->
              let* v = eval_binop a o b in
              unchanged { s with stack = v :: stack }
          | _ -> Error "stack underflow")
      | UnOp o -> (
          match s.stack with
          | v :: stack ->
              let* v = eval_unop o v in
              unchanged { s with stack = v :: stack }
          | _ -> Error "stack underflow")
      | Ap -> (
          match s.stack with
          | arg :: fun_value :: stack -> (
              let frame = { stack; env = s.env; control = s.control } in
              match fun_value with
              | VFun (x, code, env) ->
                  unchanged
                    {
                      s with
                      stack = [];
                      env = (x, arg) :: env;
                      control = code;
                      dump = frame :: s.dump;
                    }
              | VRec (f, x, code, env) as r ->
                  unchanged
                    {
                      s with
                      stack = [];
                      env = (x, arg) :: (f, r) :: env;
                      control = code;
                      dump = frame :: s.dump;
                    }
              | v ->
                  Error ("application expects a closure, found " ^ value_text v)
              )
          | _ -> Error "stack underflow")
      | Ret -> (
          match (s.stack, s.dump) with
          | [], _ -> Error "stack underflow"
          | _, [] -> Error "ret without a caller frame"
          | v :: _, frame :: dump ->
              unchanged
                {
                  s with
                  stack = v :: frame.stack;
                  env = frame.env;
                  control = frame.control;
                  dump;
                })
      | If (t, e) -> (
          match s.stack with
          | VBool b :: stack ->
              unchanged
                { s with stack; control = (if b then t else e) @ s.control }
          | [] -> Error "stack underflow"
          | v :: _ ->
              Error ("if expects a boolean condition, found " ^ value_text v)))

let step s () =
  match (s.control, s.dump, s.stack) with
  | [], [], [] -> Error "empty stack at halt"
  | [], [], v :: _ ->
      let* n = as_nat v in
      Ok (Utils_runtime.Machine.Halt n)
  | _ ->
      let* next, output = step_once s in
      Ok (Utils_runtime.Machine.Continue { next; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  let open Utils_runtime.Render in
  state
    [
      text ~title:"code" (Code.print s.code);
      text_table ~title:"status" ~columns:[ "field"; "value" ]
        [
          [
            "next";
            (match s.control with
            | [] -> "ret"
            | i :: _ -> Yojson.Safe.to_string (Code.instr_json i));
          ];
          [ "dump_depth"; string_of_int (List.length s.dump) ];
        ];
      indexed_table ~title:"stack" ~columns:[ "depth"; "value" ] value_text
        s.stack;
      keyed_table ~key_text:Utils_names.Identifier.to_string ~title:"env"
        ~columns:[ "name"; "value" ] value_text (List.rev s.env);
      indexed_table ~title:"control" ~columns:[ "pc"; "instr" ]
        (fun i -> Yojson.Safe.to_string (Code.instr_json i))
        s.control;
      text_table ~title:"dump"
        ~columns:[ "depth"; "stack"; "env"; "control" ]
        (List.mapi
           (fun i (f : dump_frame) ->
             [
               string_of_int i;
               string_of_int (List.length f.stack);
               string_of_int (List.length f.env);
               string_of_int (List.length f.control);
             ])
           s.dump);
    ]

module Ainput = Pcf_shared.Io.Ainput
module Rinput = Utils_codec.Text.Unit
module Routput = Pcf_shared.Io.Routput
module Foutput = Pcf_shared.Io.Foutput
