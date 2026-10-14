include Runtime_types
module Code = Parser.Code
module L = Utils_base.Error
open Typed_store
open Scope
open Validation
open Control

let make code input =
  L.protect (fun () ->
      validate code;
      let statics =
        List.fold_left
          (fun ss (d : static_decl) ->
            Utils_base.Assoc.set d.name
              { ty = d.ty; value = default_value d.ty }
              ss)
          [] code.statics
      in
      let s =
        {
          code;
          current_function = "main";
          exec = Ctrl Next;
          env = [];
          store = { statics; heap = []; locals = [] };
          inner_k = [];
          fn_k = [];
          next_local_id = Local_id.Supply.empty;
          next_heap_id = Heap_id.Supply.empty;
        }
      in
      enter_function s "main" (List.map (fun n -> Number_value n) input))

let step s () =
  L.protect (fun () ->
      let continue next =
        Utils_runtime.Machine.Continue { next; output = "" }
      in
      match s.exec with
      | Eval stmt -> continue (step_eval s stmt)
      | Ctrl control ->
          if s.fn_k = [] && s.inner_k = [] && control = Next && s.env = [] then
            failwith "main finished without return";
          continue (step_ctrl s control)
      | Ret values ->
          if s.fn_k = [] then
            match values with
            | [ Number_value n ] -> Utils_runtime.Machine.Halt n
            | [ _ ] -> failwith "main must return a natural number"
            | _ -> failwith "main must return exactly one value"
          else continue (step_ret s values))

let snapshot = Snapshot.encode
let restore = Snapshot.decode
let render = View.render

module Ainput = Parser.Ainput
module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.String
module Foutput = Utils_codec.Text.Natural
