open Utils

type entry = Model of (module Machine.S) | Compiler of Compiler.api

let entries =
  [
    ("example_counter", Model (module Example_counter));
    ("lambda_calculus", Model (module Lambda_calculus));
    ("goto_lang", Model (module Goto_lang));
    ("recursive_function", Model (module Recursive_function));
    ("example_counter-example_counter", Compiler Example_counter.Compiler.api);
    ("recursive_function-goto_lang", Compiler Recursive_function_goto_lang.api);
    ( "recursive_function-lambda_calculus",
      Compiler Recursive_function_lambda_calculus.api );
    ("flow_ir", Model (module Flow_ir.Machine));
    ("expr_lang", Model (module Structured_lang.Expr_lang.Machine));
    ("proc_lang", Model (module Structured_lang.Proc_lang.Machine));
    ("internal_ctrl", Model (module Structured_lang.Internal_ctrl.Machine));
    ("fn_ptr", Model (module Structured_lang.Fn_ptr.Machine));
    ("mini_prog", Model (module Structured_lang.Mini_prog.Machine));
    ("expr_to_flow_ir", Compiler Structured_lang.Expr_lang_compiler.api);
    ( "internal_ctrl_to_flow_ir",
      Compiler Structured_lang.Internal_ctrl_compiler.api );
    ("proc_to_flow_ir", Compiler Structured_lang.Proc_lang_compiler.api);
    ("pcf-expr", Model (module Pcf.Expr_machine));
    ("pcf-expr-stmt-cek", Model (module Pcf.Expr_stmt_machine));
    ("pcf-cek", Model (module Pcf.Cek_machine));
    ("pcf-secd", Model (module Pcf.Secd_machine));
    ("pcf-expr-cek", Compiler Pcf.Compiler.Expr_to_cek.api);
    ("pcf-expr-secd", Compiler Pcf.Compiler.Expr_to_secd.api);
    ("lambda_continuations", Model (module Lambda_continuations.No_ext));
    ("logic_circuit", Model (module Logic_circuit.Machine));
    ("turing_machine", Model (module Turing_machine.Machine));
    ( "rec_tm_ir",
      Model (module Recursive_function_turing_machine.Rec_tm_ir.Machine) );
    ( "rec_tm_ir_jump",
      Model (module Recursive_function_turing_machine.Rec_tm_ir_jump.Machine) );
    ( "recursive_function-rec_tm_ir",
      Compiler Recursive_function_turing_machine.Rec_to_ir.api );
    ( "rec_tm_ir-rec_tm_ir_jump",
      Compiler Recursive_function_turing_machine.Compile_jump.api );
    ( "rec_tm_ir_jump-turing_machine",
      Compiler Recursive_function_turing_machine.Compile_tm.api );
    ( "recursive_function-turing_machine",
      Compiler Recursive_function_turing_machine.Rec_to_tm.api );
    ("tiny_isa", Model (module Tiny_isa.Machine));
    ("symbolic_asm", Model (module Tiny_isa.Symbolic_asm.Machine));
    ("cfg_vreg", Model (module Tiny_isa.Cfg_vreg.Machine));
    ("coroutine", Model (module Async_await.Machine));
    ("strarr", Model (module While_with_data.Strarr.Machine));
    ("strtree", Model (module While_with_data.Strtree.Machine));
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

let library name =
  if String.starts_with ~prefix:"pcf-" name then "pcf"
  else if
    List.mem name
      [
        "expr_lang";
        "proc_lang";
        "internal_ctrl";
        "fn_ptr";
        "mini_prog";
        "expr_to_flow_ir";
        "internal_ctrl_to_flow_ir";
        "proc_to_flow_ir";
      ]
  then "structured_lang"
  else if
    List.mem name
      [
        "rec_tm_ir";
        "rec_tm_ir_jump";
        "recursive_function-rec_tm_ir";
        "rec_tm_ir-rec_tm_ir_jump";
        "rec_tm_ir_jump-turing_machine";
        "recursive_function-turing_machine";
      ]
  then "recursive_function_turing_machine"
  else if List.mem name [ "tiny_isa"; "symbolic_asm"; "cfg_vreg" ] then
    "tiny_isa"
  else if List.mem name [ "strarr"; "strtree" ] then "while_with_data"
  else if name = "coroutine" then "async_await"
  else if name = "example_counter-example_counter" then "example_counter"
  else if name = "recursive_function-goto_lang" then
    "recursive_function_goto_lang"
  else if name = "recursive_function-lambda_calculus" then
    "recursive_function_lambda_calculus"
  else name

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
