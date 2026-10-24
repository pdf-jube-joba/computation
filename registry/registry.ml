module Machine = Utils_runtime.Machine

type implementation =
  | Model of (module Machine.S)
  | Compiler of Compiler_text_api.api

type entry = {
  name : string;
  library : string;
  implementation : implementation;
}

let entries =
  [
    {
      name = "interaction_combinators";
      library = "interaction_combinators";
      implementation = Model (module Interaction_combinators.Main);
    };
    {
      name = "cellular_automaton";
      library = "cellular_automaton";
      implementation = Model (module Cellular_automaton.Main);
    };
    {
      name = "game_of_life";
      library = "cellular_automaton";
      implementation = Model (module Cellular_automaton.Life);
    };
    {
      name = "brainfuck";
      library = "brainfuck";
      implementation = Model (module Brainfuck.Main);
    };
    {
      name = "ski_calculus";
      library = "ski_calculus";
      implementation = Model (module Ski_calculus.Main);
    };
    {
      name = "p_double_prime";
      library = "p_double_prime";
      implementation = Model (module P_double_prime.Main);
    };
    {
      name = "pure_lisp";
      library = "pure_lisp";
      implementation = Model (module Pure_lisp.Main);
    };
    {
      name = "stg";
      library = "stg";
      implementation = Model (module Stg.Main);
    };
    {
      name = "example_counter";
      library = "example_counter";
      implementation = Model (module Example_counter.Main);
    };
    {
      name = "lambda_calculus";
      library = "lambda_calculus";
      implementation = Model (module Lambda_calculus.Main);
    };
    {
      name = "goto_lang";
      library = "goto_lang";
      implementation = Model (module Goto_lang.Main);
    };
    {
      name = "recursive_function";
      library = "recursive_function";
      implementation = Model (module Recursive_function.Main);
    };
    {
      name = "example_counter-example_counter";
      library = "example_counter_to_example_counter";
      implementation =
        Compiler
          (Compiler_text_api.make
             (module Example_counter_to_example_counter.Main));
    };
    {
      name = "recursive_function-goto_lang";
      library = "recursive_function_to_goto_lang";
      implementation =
        Compiler
          (Compiler_text_api.make (module Recursive_function_to_goto_lang.Main));
    };
    {
      name = "recursive_function-lambda_calculus";
      library = "recursive_function_to_lambda_calculus";
      implementation =
        Compiler
          (Compiler_text_api.make
             (module Recursive_function_to_lambda_calculus.Main));
    };
    {
      name = "core_machine";
      library = "core_machine";
      implementation = Model (module Core_machine.Main);
    };
    {
      name = "flow_ir_to_core_machine";
      library = "flow_ir_to_core_machine";
      implementation =
        Compiler (Compiler_text_api.make (module Flow_ir_to_core_machine.Main));
    };
    {
      name = "flow_ir";
      library = "flow_ir";
      implementation = Model (module Flow_ir.Main);
    };
    {
      name = "expr_lang";
      library = "expr_lang";
      implementation = Model (module Expr_lang.Main);
    };
    {
      name = "proc_lang";
      library = "proc_lang";
      implementation = Model (module Proc_lang.Main);
    };
    {
      name = "internal_ctrl";
      library = "internal_ctrl";
      implementation = Model (module Internal_ctrl.Main);
    };
    {
      name = "fn_ptr";
      library = "fn_ptr";
      implementation = Model (module Fn_ptr.Main);
    };
    {
      name = "mini_prog";
      library = "mini_prog";
      implementation = Model (module Mini_prog.Main);
    };
    {
      name = "expr_to_flow_ir";
      library = "expr_lang_to_flow_ir";
      implementation =
        Compiler (Compiler_text_api.make (module Expr_lang_to_flow_ir.Main));
    };
    {
      name = "internal_ctrl_to_flow_ir";
      library = "internal_ctrl_to_flow_ir";
      implementation =
        Compiler (Compiler_text_api.make (module Internal_ctrl_to_flow_ir.Main));
    };
    {
      name = "proc_to_flow_ir";
      library = "proc_lang_to_flow_ir";
      implementation =
        Compiler (Compiler_text_api.make (module Proc_lang_to_flow_ir.Main));
    };
    {
      name = "pcf-expr";
      library = "pcf_expr";
      implementation = Model (module Pcf_expr.Main);
    };
    {
      name = "pcf-expr-stmt-cek";
      library = "pcf_expr_stmt";
      implementation = Model (module Pcf_expr_stmt.Main);
    };
    {
      name = "pcf-cek";
      library = "pcf_cek";
      implementation = Model (module Pcf_cek.Main);
    };
    {
      name = "pcf-secd";
      library = "pcf_secd";
      implementation = Model (module Pcf_secd.Main);
    };
    {
      name = "pcf-expr-cek";
      library = "pcf_expr_to_pcf_cek";
      implementation =
        Compiler (Compiler_text_api.make (module Pcf_expr_to_pcf_cek.Main));
    };
    {
      name = "pcf-expr-secd";
      library = "pcf_expr_to_pcf_secd";
      implementation =
        Compiler (Compiler_text_api.make (module Pcf_expr_to_pcf_secd.Main));
    };
    {
      name = "lambda_continuations";
      library = "lambda_continuations_no_ext";
      implementation = Model (module Lambda_continuations_no_ext.Main);
    };
    {
      name = "logic_circuit";
      library = "logic_circuit";
      implementation = Model (module Logic_circuit.Main);
    };
    {
      name = "turing_machine";
      library = "one_tape_turing_machine";
      implementation = Model (module One_tape_turing_machine.Main);
    };
    {
      name = "one_tape_turing_machine";
      library = "one_tape_turing_machine";
      implementation = Model (module One_tape_turing_machine.Main);
    };
    {
      name = "multi_tape_turing_machine";
      library = "multi_tape_turing_machine";
      implementation = Model (module Multi_tape_turing_machine.Main);
    };
    {
      name = "multi_tape_turing_machine-one_tape_turing_machine";
      library = "multi_tape_turing_machine_to_one_tape_turing_machine";
      implementation =
        Compiler
          (Compiler_text_api.make
             (module Multi_tape_turing_machine_to_one_tape_turing_machine.Main));
    };
    {
      name = "rec_tm_ir";
      library = "rec_tm_ir";
      implementation = Model (module Rec_tm_ir.Main);
    };
    {
      name = "rec_tm_ir_jump";
      library = "rec_tm_ir_jump";
      implementation = Model (module Rec_tm_ir_jump.Main);
    };
    {
      name = "recursive_function-rec_tm_ir";
      library = "recursive_function_to_rec_tm_ir";
      implementation =
        Compiler
          (Compiler_text_api.make (module Recursive_function_to_rec_tm_ir.Main));
    };
    {
      name = "rec_tm_ir-rec_tm_ir_jump";
      library = "rec_tm_ir_to_rec_tm_ir_jump";
      implementation =
        Compiler
          (Compiler_text_api.make (module Rec_tm_ir_to_rec_tm_ir_jump.Main));
    };
    {
      name = "rec_tm_ir_jump-turing_machine";
      library = "rec_tm_ir_jump_to_turing_machine";
      implementation =
        Compiler
          (Compiler_text_api.make
             (module Rec_tm_ir_jump_to_turing_machine.Main));
    };
    {
      name = "recursive_function-turing_machine";
      library = "recursive_function_to_turing_machine";
      implementation =
        Compiler
          (Compiler_text_api.make
             (module Recursive_function_to_turing_machine.Main));
    };
    {
      name = "tiny_isa";
      library = "tiny_isa";
      implementation = Model (module Tiny_isa.Main);
    };
    {
      name = "symbolic_asm";
      library = "symbolic_asm";
      implementation = Model (module Symbolic_asm.Main);
    };
    {
      name = "cfg_vreg";
      library = "cfg_vreg";
      implementation = Model (module Cfg_vreg.Main);
    };
    {
      name = "coroutine";
      library = "coroutine";
      implementation = Model (module Coroutine.Main);
    };
    {
      name = "strarr";
      library = "strarr";
      implementation = Model (module Strarr.Main);
    };
    {
      name = "strtree";
      library = "strtree";
      implementation = Model (module Strtree.Main);
    };
  ]

let find name =
  match List.find_opt (fun entry -> entry.name = name) entries with
  | Some entry -> Ok entry.implementation
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

let manifest () =
  `List
    (List.map
       (fun entry ->
         `Assoc
           [
             ("name", `String entry.name);
             ("library", `String entry.library);
             ( "kind",
               `String
                 (match entry.implementation with
                 | Model _ -> "model"
                 | Compiler _ -> "compiler") );
           ])
       entries)
