open Syntax

let ( let* ) = Result.bind

type instr =
  | PushNat of N.t
  | PushBool of bool
  | PushUnit
  | Print of N.t
  | Acc of Utils.Identifier.t
  | MkCls of Utils.Identifier.t * instr list
  | MkRec of Utils.Identifier.t * Utils.Identifier.t * instr list
  | BinOp of binop
  | UnOp of unop
  | Ap
  | Ret
  | If of instr list * instr list
[@@deriving yojson]

type value =
  | VNat of N.t
  | VBool of bool
  | VUnit
  | VFun of Utils.Identifier.t * instr list * env
  | VRec of Utils.Identifier.t * Utils.Identifier.t * instr list * env

and env = (Utils.Identifier.t * value) list [@@deriving yojson]

type dump_frame = { stack : value list; env : env; control : instr list }
[@@deriving yojson]

type code = instr list [@@deriving yojson]
type ainput = N.t list
type rinput = unit
type routput = print_effect
type foutput = N.t

type state = {
  code : code;
  stack : value list;
  env : env;
  control : instr list;
  dump : dump_frame list;
}
[@@deriving yojson]

(* Public code retains Rust serde's externally tagged instruction syntax. *)
let json_nat n =
  match N.to_int n with Ok i -> `Int i | Error _ -> `Intlit (N.to_string n)

let rec instr_json = function
  | PushNat n -> `Assoc [ ("PushNat", json_nat n) ]
  | PushBool b -> `Assoc [ ("PushBool", `Bool b) ]
  | PushUnit -> `String "PushUnit"
  | Print n -> `Assoc [ ("Print", json_nat n) ]
  | Acc x -> `Assoc [ ("Acc", `String x) ]
  | MkCls (x, c) ->
      `Assoc
        [ ("MkCls", `Assoc [ ("param", `String x); ("code", code_json c) ]) ]
  | MkRec (f, x, c) ->
      `Assoc
        [
          ( "MkRec",
            `Assoc
              [
                ("name", `String f); ("param", `String x); ("code", code_json c);
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

and code_json c = `List (List.map instr_json c)

let get = function Ok x -> x | Error e -> failwith e

let natural_json = function
  | `Int n when n >= 0 -> N.of_int n
  | `Intlit s -> get (N.of_string s)
  | _ -> failwith "expected natural number"

let identifier_json j = get (Utils.Identifier.make (Utils.Json.string j))

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
        ( identifier_json (Utils.Json.field "param" j),
          code_from_json (Utils.Json.field "code" j) )
  | `Assoc [ ("MkRec", j) ] ->
      MkRec
        ( identifier_json (Utils.Json.field "name" j),
          identifier_json (Utils.Json.field "param" j),
          code_from_json (Utils.Json.field "code" j) )
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
        ( code_from_json (Utils.Json.field "then_code" j),
          code_from_json (Utils.Json.field "else_code" j) )
  | _ -> failwith "invalid SECD instruction"

and code_from_json j = List.map instr_from_json (Utils.Json.list j)

let parse_code text =
  let* j = Utils.Json.decode text in
  Utils.Json.protect (fun () -> code_from_json j)

let print_code c = Yojson.Safe.pretty_to_string (code_json c)
let parse_ainput = Syntax.parse_ainput
let parse_rinput = Utils.Machine.empty_input

let make code inputs =
  Ok
    {
      code;
      stack = [];
      env = [];
      control = code @ List.concat_map (fun n -> [ PushNat n; Ap ]) inputs;
      dump = [];
    }

let eval_binop a op b =
  match (op, a, b) with
  | Add, VNat a, VNat b -> Ok (VNat (N.add a b))
  | Sub, VNat a, VNat b when N.compare a b >= 0 -> Ok (VNat (N.sub a b))
  | Sub, VNat _, VNat _ -> Error "operator '-' expects lhs >= rhs over naturals"
  | And, VBool a, VBool b -> Ok (VBool (a && b))
  | Add, _, _ -> Error "operator '+' expects naturals"
  | Sub, _, _ -> Error "operator '-' expects naturals"
  | And, _, _ -> Error "operator '&&' expects booleans"

let eval_unop op value =
  match (op, value) with
  | Inc, VNat n -> Ok (VNat (N.succ n))
  | Dec, VNat n when not (N.is_zero n) -> Ok (VNat (N.pred n))
  | Dec, VNat _ -> Error "operator 'dec' expects a positive natural"
  | Not, VBool b -> Ok (VBool (not b))
  | Inc, _ -> Error "operator 'inc' expects a natural"
  | Dec, _ -> Error "operator 'dec' expects a natural"
  | Not, _ -> Error "operator 'not' expects a boolean"

let as_nat = function
  | VNat n -> Ok n
  | _ -> Error "final value is not a natural number"

let value_text = function
  | VNat n -> N.to_string n
  | VBool b -> if b then "#true" else "#false"
  | VUnit -> "#unit"
  | VFun (x, c, _) -> "closure " ^ x ^ " " ^ print_code c
  | VRec (f, x, c, _) -> "recursive closure " ^ f ^ " " ^ x ^ " " ^ print_code c

let lookup_env env x =
  match List.assoc_opt x env with
  | Some v -> Ok v
  | None -> Error ("unbound variable: " ^ x)

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
      Ok (Utils.Machine.Halt n)
  | _ ->
      let* next, output = step_once s in
      Ok (Utils.Machine.Continue { next; output })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  let table = Cek_machine.table in
  let indexed f xs = List.mapi (fun i x -> [ string_of_int i; f x ]) xs in
  Utils.Render.state
    [
      Utils.Render.text ~title:"code" (print_code s.code);
      table "status" [ "field"; "value" ]
        [
          [
            "next";
            (match s.control with
            | [] -> "ret"
            | i :: _ -> Yojson.Safe.to_string (instr_json i));
          ];
          [ "dump_depth"; string_of_int (List.length s.dump) ];
        ];
      table "stack" [ "depth"; "value" ] (indexed value_text s.stack);
      table "env" [ "name"; "value" ]
        (List.rev_map (fun (x, v) -> [ x; value_text v ]) s.env);
      table "control" [ "pc"; "instr" ]
        (indexed (fun i -> Yojson.Safe.to_string (instr_json i)) s.control);
      table "dump"
        [ "depth"; "stack"; "env"; "control" ]
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

let routput_json = Syntax.routput_json
let foutput_json = Syntax.foutput_json

let rec compile_expr (e : expr) =
  match e with
  | Nat n -> [ PushNat n ]
  | Bool b -> [ PushBool b ]
  | Unit -> [ PushUnit ]
  | Syntax.Print n -> [ Print n ]
  | Var x -> [ Acc x ]
  | Syntax.BinOp (a, o, b) -> compile_expr a @ compile_expr b @ [ BinOp o ]
  | Syntax.UnOp (o, e) -> compile_expr e @ [ UnOp o ]
  | Fun (x, b) -> [ MkCls (x, compile_expr b @ [ Ret ]) ]
  | Rec (f, x, b) -> [ MkRec (f, x, compile_expr b @ [ Ret ]) ]
  | App (f, a) -> compile_expr f @ compile_expr a @ [ Ap ]
  | Syntax.If (c, t, e) ->
      compile_expr c @ [ If (compile_expr t, compile_expr e) ]
  | Block _ ->
      invalid_arg "SECD compiler accepts the direct expression language"
