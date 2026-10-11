let discard parse text = Result.map (fun _ -> ()) (parse text)

let ok context = function
  | Ok _ -> ()
  | Error message -> failwith (context ^ ": " ^ message)

let error context = function
  | Error _ -> ()
  | Ok _ -> failwith (context ^ ": accepted an invalid identifier")

module R = Recursive_function_turing_machine
module S = Structured_lang
module A = Tiny_isa.Symbolic_asm
module C = Tiny_isa.Cfg_vreg

let parsers =
  [
    ("identifier", discard Utils.Identifier.make, Fun.id);
    ("GOTO variable", discard Goto_lang.parse_code, fun n -> "inc " ^ n ^ ";");
    ( "recursive function",
      discard Recursive_function.parse_code,
      fun n -> "let " ^ n ^ " = SUCC. " ^ n );
    ( "lambda variable",
      discard Lambda_calculus.parse_code,
      fun n -> "\\" ^ n ^ ". " ^ n );
    ( "lambda continuation variable",
      discard Lambda_continuations.Lambda.parse_code,
      fun n -> "fun " ^ n ^ " => " ^ n );
    ( "natural lambda variable",
      discard Lambda_continuations.Parse.lam_nat,
      fun n -> "fun " ^ n ^ " => " ^ n );
    ( "coroutine function",
      discard Async_await.Coroutine.parse_code,
      fun n -> "fn main {call " ^ n ^ "} fn " ^ n ^ " {yield}" );
    ( "coroutine variable",
      discard Async_await.Coroutine.parse_code,
      fun n -> "fn main {" ^ n ^ " <- 1}" );
    ( "coroutine task ID",
      discard Async_await.Coroutine.parse_code,
      fun n -> "fn main {run main -> $" ^ n ^ "}" );
    ( "coroutine input",
      discard Async_await.Coroutine.parse_env,
      fun n -> n ^ " = 1" );
    ("HDL enum", discard My_hdl.parse_value, fun n -> "<" ^ n ^ ">");
    ( "circuit pin",
      discard Logic_circuit.parse_code,
      fun n -> "graph: main {in {" ^ n ^ "} out {}}" );
    ( "circuit input",
      discard Logic_circuit.parse_rinput,
      fun n -> n ^ ".IN=#true" );
    ( "array variable",
      discard While_with_data.Strarr.parse_code,
      fun n -> n ^ " := []" );
    ( "tree variable",
      discard While_with_data.Strtree.parse_code,
      fun n -> n ^ " := atom 'a'" );
    ( "array input",
      discard While_with_data.Strarr.parse_ainput,
      fun n -> n ^ " = []" );
    ( "tree input",
      discard While_with_data.Strtree.parse_ainput,
      fun n -> n ^ " = atom 'a'" );
    ( "PCF variable",
      discard Pcf.Syntax.parse_code,
      fun n -> "fun " ^ n ^ " => " ^ n );
    ("expression variable", discard S.Expr_lang.parse_code, fun n -> n ^ " := 1");
    ( "procedure parameter",
      discard S.Proc_lang.parse_code,
      fun n -> "static ; main(" ^ n ^ ") [local temp return]" );
    ( "loop variable",
      discard S.Internal_ctrl.parse_code,
      fun n -> "static " ^ n ^ "; " ^ n ^ " := 1" );
    ( "function pointer parameter",
      discard S.Fn_ptr.parse_code,
      fun n -> "fn main(" ^ n ^ "){return}" );
    ( "mini program static",
      discard S.Mini_prog.parse_code,
      fun n -> "static " ^ n ^ ":#num fn main(){return 0}" );
    ("structured input", discard S.Common.parse_env, fun n -> n ^ " = 1");
    ("IR label", discard Flow_ir.parse_code, fun n -> ":main{:" ^ n ^ "{halt;}}");
    ( "IR register",
      discard Flow_ir.parse_code,
      fun n -> ":main{:entry{%" ^ n ^ ":=#0;halt;}}" );
    ( "IR static",
      discard Flow_ir.parse_code,
      fun n -> "@" ^ n ^ " 0 :main{:entry{halt;}}" );
    ("IR input", discard Flow_ir.parse_ainput, fun n -> n ^ " = 1");
    ("IR label value", discard Flow_ir.parse_flow_value, fun n -> ":" ^ n);
    ("IR static key", discard Flow_ir.parse_flow_value, fun n -> "k:@" ^ n);
    ("assembly label", discard A.parse_code, fun n -> "@" ^ n ^ ":\nhalt");
    ( "assembly label operand",
      discard A.parse_code,
      fun n -> "ldi %1 @" ^ n ^ "\nhalt" );
    ( "assembly constant",
      discard A.parse_code,
      fun n -> ".equ " ^ n ^ " #1\nldi %1 " ^ n ^ "\nhalt" );
    ( "CFG label",
      discard C.parse_code,
      fun n -> "@" ^ n ^ " {\ngoto @" ^ n ^ ";\n}" );
    ( "CFG label operand",
      discard C.parse_code,
      fun n -> "@main {\ngoto @" ^ n ^ ";\n}" );
    ( "CFG static",
      discard C.parse_code,
      fun n -> "@" ^ n ^ " 1\n@main {\ngoto @main;\n}" );
    ("Turing state", discard Turing_machine.parse_state, Fun.id);
    ("Turing sign", discard Turing_machine.parse_sign, Fun.id);
    ( "recursive IR variable",
      discard R.Rec_tm_ir.parse_code,
      fun n -> "alphabet:(a) fn main {label entry:{" ^ n ^ ":=const a}}" );
    ( "recursive IR function",
      discard R.Rec_tm_ir.parse_code,
      fun n ->
        "alphabet:(a) fn " ^ n ^ " {label entry:{}} fn main {label entry:{call "
        ^ n ^ "}}" );
    ( "recursive IR label",
      discard R.Rec_tm_ir.parse_code,
      fun n -> "alphabet:(a) fn main {label " ^ n ^ ":{jump " ^ n ^ "}}" );
    ( "jump IR variable",
      discard R.Rec_tm_ir_jump.parse_code,
      fun n -> "alphabet:(a) " ^ n ^ ":=const a" );
  ]

let () =
  List.iter
    (fun (context, parse, source) ->
      List.iter
        (fun name -> ok (context ^ " / " ^ name) (parse (source name)))
        [ "foo_bar"; "_"; "_temp_1"; "A0_Z" ];
      List.iter
        (fun name -> error (context ^ " / " ^ name) (parse (source name)))
        [ "foo-bar"; "foo'"; "0foo"; "123"; "foo.bar"; "猫" ])
    parsers;
  (* JSON input and snapshot decoders use the same identifier rules. *)
  List.iter
    (fun decode ->
      ok "valid JSON name" (decode "foo_bar");
      error "old JSON name" (decode "foo-bar"))
    [
      (fun n ->
        discard My_hdl.Value.of_yojson
          (My_hdl.Value.to_yojson (My_hdl.Value.Enume n)));
      (fun n ->
        discard Goto_lang.command_of_yojson
          (Goto_lang.command_to_yojson (Goto_lang.Inc n)));
      (fun n ->
        discard Async_await.Coroutine.global_env_of_yojson
          (Async_await.Coroutine.global_env_to_yojson
             { vars = [ (n, "1") ]; task_ids = [] }));
      (fun n ->
        discard Pcf.Syntax.expr_of_yojson
          (Pcf.Syntax.expr_to_yojson (Pcf.Syntax.Var n)));
      (fun n ->
        discard Lambda_continuations.Lambda.term_of_yojson
          (Lambda_continuations.Lambda.term_to_yojson
             (Lambda_continuations.Lambda.Var n)));
      (fun n ->
        discard Flow_ir.flow_value_of_yojson
          (Flow_ir.flow_value_to_yojson (Flow_ir.Code_label n)));
      (fun n ->
        discard A.raw_value_of_yojson (A.raw_value_to_yojson (A.Label n)));
      (fun n ->
        discard C.addr_expr_of_yojson (C.addr_expr_to_yojson (C.Label n)));
      (fun n ->
        discard While_with_data.Strarr.aexp_of_yojson
          (While_with_data.Strarr.aexp_to_yojson (While_with_data.Strarr.Var n)));
      (fun n ->
        discard While_with_data.Strtree.texp_of_yojson
          (While_with_data.Strtree.texp_to_yojson
             (While_with_data.Strtree.Var n)));
      (fun n ->
        discard S.Common.env_of_yojson (S.Common.env_to_yojson [ (n, "1") ]));
      (fun n ->
        discard S.Mini_prog_syntax.value_of_yojson
          (S.Mini_prog_syntax.value_to_yojson
             (S.Mini_prog_syntax.FnPtr (Some n))));
      (fun n ->
        discard Turing_machine.sign_of_yojson
          (Turing_machine.sign_to_yojson (Some n)));
      (fun n ->
        discard R.Rec_tm_ir.lvalue_of_yojson
          (R.Rec_tm_ir.lvalue_to_yojson (R.Rec_tm_ir.Var n)));
      (fun n ->
        discard Lambda_calculus.restore
          (Lambda_calculus.snapshot (Lambda_calculus.Var { id = 0; name = n })));
    ];
  ok "blank Turing sign" (Turing_machine.parse_sign "-");
  ok "blank recursive IR sign"
    (R.Rec_tm_ir_jump.parse_code "alphabet:(-) @:=const -");
  ok "circuit gate name"
    (Logic_circuit.parse_code "graph: main {in {} out {} A, NOT_F {}}");
  error "old circuit gate name"
    (Logic_circuit.parse_code "graph: main {in {} out {} A, NOT-F {}}");
  error "old HDL name" (My_hdl.parse_value "<->");
  ok "array keyword"
    (While_with_data.Strarr.parse_code "if is_empty x then Nop end");
  error "old array keyword"
    (While_with_data.Strarr.parse_code "if is-empty x then Nop end");
  ok "tree keyword"
    (While_with_data.Strtree.parse_code "ifeq is_atom x then Nop end");
  error "old tree keyword"
    (While_with_data.Strtree.parse_code "ifeq is-atom x then Nop end");
  ok "pointer literal"
    (S.Fn_ptr.parse_code "fn main(){assign x := #null_ptr;return}");
  error "old pointer literal"
    (S.Fn_ptr.parse_code "fn main(){assign x := #null-ptr;return}");
  ok "mini program literals"
    (S.Mini_prog.parse_code
       "fn main(){if #null_ptr #null_ptr == return 1;return 0}");
  error "old mini program literal"
    (S.Mini_prog.parse_code
       "fn main(){if #null-ptr #null-ptr == return 1;return 0}");
  assert (
    Async_await.Coroutine.parse_code "fn main {x<-a-b}"
    = Async_await.Coroutine.parse_code "fn main {x <- a - b}");
  print_endline "Unified identifier tests passed"
