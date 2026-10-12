open Utils

type entry = Model of (module Machine.S) | Compiler of Compiler.api

let entries =
  [
    ("example_counter", Model (module Example_counter.Main));
    ("lambda_calculus", Model (module Lambda_calculus.Main));
    ("goto_lang", Model (module Goto_lang.Main));
    ("recursive_function", Model (module Recursive_function.Main));
    ( "example_counter-example_counter",
      Compiler Example_counter_to_example_counter.Main.api );
    ( "recursive_function-goto_lang",
      Compiler Recursive_function_to_goto_lang.Main.api );
    ( "recursive_function-lambda_calculus",
      Compiler Recursive_function_to_lambda_calculus.Main.api );
    ("flow_ir", Model (module Flow_ir.Main));
    ("expr_lang", Model (module Expr_lang.Main));
    ("proc_lang", Model (module Proc_lang.Main));
    ("internal_ctrl", Model (module Internal_ctrl.Main));
    ("fn_ptr", Model (module Fn_ptr.Main));
    ("mini_prog", Model (module Mini_prog.Main));
    ("expr_to_flow_ir", Compiler Expr_lang_to_flow_ir.Main.api);
    ("internal_ctrl_to_flow_ir", Compiler Internal_ctrl_to_flow_ir.Main.api);
    ("proc_to_flow_ir", Compiler Proc_lang_to_flow_ir.Main.api);
    ("pcf-expr", Model (module Pcf_expr.Main));
    ("pcf-expr-stmt-cek", Model (module Pcf_expr_stmt.Main));
    ("pcf-cek", Model (module Pcf_cek.Main));
    ("pcf-secd", Model (module Pcf_secd.Main));
    ("pcf-expr-cek", Compiler Pcf_expr_to_pcf_cek.Main.api);
    ("pcf-expr-secd", Compiler Pcf_expr_to_pcf_secd.Main.api);
    ("lambda_continuations", Model (module Lambda_continuations_no_ext.Main));
    ("logic_circuit", Model (module Logic_circuit.Main));
    ("turing_machine", Model (module Turing_machine.Main));
    ("rec_tm_ir", Model (module Rec_tm_ir.Main));
    ("rec_tm_ir_jump", Model (module Rec_tm_ir_jump.Main));
    ( "recursive_function-rec_tm_ir",
      Compiler Recursive_function_to_rec_tm_ir.Main.api );
    ("rec_tm_ir-rec_tm_ir_jump", Compiler Rec_tm_ir_to_rec_tm_ir_jump.Main.api);
    ( "rec_tm_ir_jump-turing_machine",
      Compiler Rec_tm_ir_jump_to_turing_machine.Main.api );
    ( "recursive_function-turing_machine",
      Compiler Recursive_function_to_turing_machine.Main.api );
    ("tiny_isa", Model (module Tiny_isa.Main));
    ("symbolic_asm", Model (module Symbolic_asm.Main));
    ("cfg_vreg", Model (module Cfg_vreg.Main));
    ("coroutine", Model (module Coroutine.Main));
    ("strarr", Model (module Strarr.Main));
    ("strtree", Model (module Strtree.Main));
  ]

let find name =
  match List.assoc_opt name entries with
  | Some entry -> Ok entry
  | None -> Error ("Unknown model/compiler: " ^ name)

let model name =
  match find name with
  | Ok (Model model) -> Ok model
  | Ok (Compiler _) -> Error (name ^ " is a compiler")
  | Error e -> Error e

let compiler name =
  match find name with
  | Ok (Compiler compiler) -> Ok compiler
  | Ok (Model _) -> Error (name ^ " is a model")
  | Error e -> Error e

let library = function
  | "example_counter-example_counter" -> "example_counter_to_example_counter"
  | "recursive_function-goto_lang" -> "recursive_function_to_goto_lang"
  | "recursive_function-lambda_calculus" ->
      "recursive_function_to_lambda_calculus"
  | "expr_to_flow_ir" -> "expr_lang_to_flow_ir"
  | "internal_ctrl_to_flow_ir" -> "internal_ctrl_to_flow_ir"
  | "proc_to_flow_ir" -> "proc_lang_to_flow_ir"
  | "pcf-expr" -> "pcf_expr"
  | "pcf-expr-stmt-cek" -> "pcf_expr_stmt"
  | "pcf-cek" -> "pcf_cek"
  | "pcf-secd" -> "pcf_secd"
  | "pcf-expr-cek" -> "pcf_expr_to_pcf_cek"
  | "pcf-expr-secd" -> "pcf_expr_to_pcf_secd"
  | "lambda_continuations" -> "lambda_continuations_no_ext"
  | "recursive_function-rec_tm_ir" -> "recursive_function_to_rec_tm_ir"
  | "rec_tm_ir-rec_tm_ir_jump" -> "rec_tm_ir_to_rec_tm_ir_jump"
  | "rec_tm_ir_jump-turing_machine" -> "rec_tm_ir_jump_to_turing_machine"
  | "recursive_function-turing_machine" ->
      "recursive_function_to_turing_machine"
  | name -> name

let manifest () =
  `List
    (List.map
       (fun (name, entry) ->
         `Assoc
           [
             ("name", `String name);
             ("library", `String (library name));
             ( "kind",
               `String
                 (match entry with
                 | Model _ -> "model"
                 | Compiler _ -> "compiler") );
           ])
       entries)
