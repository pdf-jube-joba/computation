let ( let* ) = Result.bind

include Types
module Code = Code
open Runtime_value

let make code inputs =
  Ok
    {
      code;
      stack = [];
      env = [];
      control = code @ List.concat_map (fun n -> [ PushNat n; Ap ]) inputs;
      dump = [];
    }

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

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson
  let of_yojson = state_of_yojson
end

let render s =
  let open Utils_runtime.Render in
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
    Pcf_shared.View.env value_text s.env;
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
module Rinput = Utils_data.Text.Unit
module Routput = Pcf_shared.Io.Routput
module Foutput = Pcf_shared.Io.Foutput
