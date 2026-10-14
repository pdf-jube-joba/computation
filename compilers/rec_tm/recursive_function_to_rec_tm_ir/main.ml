open Utils_runtime.Machine
open Rec_tm_ir.Main
module I = Rec_tm_ir.Main
module N = Utils_base.Number
module R = Recursive_function.Main
module T = Turing_machine.Main
module L = Utils_parsing.Parser
open Rec_tm_compiler_shared.Tape_encoding
module Function_id = Utils_names.Id.Make ()
module Function_names = Utils_names.Naming.Make (Function_id)

module Functions =
  Utils_names.Intern.Make
    (struct
      type t = I.block list

      let compare = Stdlib.compare
    end)
    (Function_id)

type builder = {
  mutable functions : func list;
  identities : Functions.t ref;
  mutable names : Function_names.t;
  mutable definitions : func Function_id.Map.t;
}

let create () =
  {
    functions = [];
    identities = ref Functions.empty;
    names = Function_names.reserve "main" Function_names.empty;
    definitions = Function_id.Map.empty;
  }

let register ctx (f : func) =
  let id = Functions.take f.blocks ctx.identities in
  match Function_id.Map.find_opt id ctx.definitions with
  | Some old -> old
  | None ->
      let name, names = Function_names.assign ~hint:f.name id ctx.names in
      ctx.names <- names;
      let f = { f with name } in
      ctx.definitions <- Function_id.Map.add id f ctx.definitions;
      ctx.functions <- ctx.functions @ [ f ];
      f

let block label body = { label; body }
let fn name blocks = { name; blocks }

let call ctx f =
  let f = register ctx f in
  Call f.name

let put s = Assign (Head, Const s)
let store name value = Assign (Var name, value)
let put_var name = Assign (Head, Var name)
let eq a b = Some { left = a; right = b }
let head_is s = eq Head (Const s)
let var_is name s = eq (Var name) (Const s)
let swap = [ store "tmp" Head; put_var "put"; store "put" (Var "tmp") ]

let move_right_till_x_n_times n =
  fn
    (Printf.sprintf "move_right_till_x_%d" n)
    (List.init n (fun i ->
         block
           (Printf.sprintf "until_x_%d" i)
           [ Rt; Break (head_is x); Continue None ]))

let move_left_till_x_n_times n =
  fn
    (Printf.sprintf "move_left_till_x_%d" n)
    (List.init n (fun i ->
         block
           (Printf.sprintf "until_x_%d" i)
           [ Lt; Break (head_is x); Continue None ]))

let call_r ctx n = call ctx (move_right_till_x_n_times n)
let call_l ctx n = call ctx (move_left_till_x_n_times n)

let concat ctx =
  fn "concat"
    [
      block "initially" [ call_r ctx 2; store "put" (Const x); put b ];
      block "loop" ([ Lt ] @ swap @ [ Break (var_is "put" x); Continue None ]);
      block "finally" [ call_l ctx 1 ];
    ]

let shift_left_x _ctx s =
  fn
    ("shift_left_put_" ^ match s with None -> "blank" | Some name -> name)
    [
      block "initially" [ store "put" Head; put s; Lt ];
      block "loop" (swap @ [ Return (var_is "put" x); Lt; Continue None ]);
    ]

let shift_left_x_n_times ctx n =
  fn
    (Printf.sprintf "shift_left_%d" n)
    (block "initial" [ call_r ctx n; store "put" (Const x); put b ]
    :: List.concat
         (List.init n (fun i ->
              [
                block (Printf.sprintf "initial_%d" i) [ Lt ];
                block
                  (Printf.sprintf "loop_%d" i)
                  (swap @ [ Break (var_is "put" x); Lt; Continue None ]);
              ])))

let copy_to_end ctx n =
  fn
    (Printf.sprintf "copy_to_end_%d" n)
    [
      block "initially" [ call_r ctx (n + 1); Rt; put x; call_l ctx (n + 2) ];
      block "main"
        [
          Rt;
          Break (head_is x);
          store "put" Head;
          put x;
          call_r ctx (n + 2);
          put_var "put";
          Rt;
          put x;
          call_l ctx (n + 2);
          put_var "put";
          Continue None;
        ];
      block "finally" [ call_l ctx 1 ];
    ]

let copy_n_times ctx n =
  let c = call ctx (copy_to_end ctx 0) in
  fn
    (Printf.sprintf "copy_%d" n)
    (List.init n (fun i ->
         block (Printf.sprintf "copy_head_%d" i) [ c; call_r ctx 1 ])
    @ List.init n (fun i ->
        block (Printf.sprintf "copy_tail_%d" i) [ call_l ctx 1 ]))

let copy ctx n = call ctx (copy_n_times ctx n)

let swap_tuple ctx =
  fn "swap_tuple"
    [
      block "initially" [ store "put" (Const x); call_r ctx 2 ];
      block "loop_1" (swap @ [ Lt; Break (var_is "put" x); Continue None ]);
      block "loop_2" (swap @ [ Lt; Break (var_is "put" x); Continue None ]);
      block "loop_3" (swap @ [ Lt; Break (var_is "put" x); Continue None ]);
      block "left_of_left"
        ([ Rt; Return (head_is x) ]
        @ swap
        @ [ call_r ctx 3; Lt; Jump ("loop_1", None) ]);
    ]

let rotate ctx n =
  if n < 2 then fn (Printf.sprintf "rotate_%d" n) []
  else
    let sw = call ctx (swap_tuple ctx) in
    fn
      (Printf.sprintf "rotate_%d" n)
      (List.init (n - 1) (fun i ->
           block (Printf.sprintf "rot_swap_%d" i) [ sw; call_r ctx 1 ])
      @ [ block "return_to_init" [ call_l ctx (n - 1) ] ])

let zero_function () =
  fn "zero_const_function" [ block "entry" [ Rt; put b; Rt; put x; Lt; Lt ] ]

let succ_function ctx =
  fn "succ_function"
    [ block "entry" [ call_r ctx 1; put l; Rt; put x; call_l ctx 1 ] ]

let aux_projection_init ctx n i =
  fn
    (Printf.sprintf "aux_proj_%d_%d" n i)
    ([ block "initial" [ Rt ] ]
    @ List.init i (fun k ->
        block
          (Printf.sprintf "flat_%d" k)
          [ put b; Rt; Break (head_is b); Continue None ])
    @ [ block "move_x" [ call_r ctx 1 ] ]
    @ List.init
        (n - i - 1)
        (fun k ->
          block
            (Printf.sprintf "flat_%d" (n - 1 - k))
            [ put b; Lt; Break (head_is b); Continue None ])
    @ [ block "push_end_x" [ put x ] ])

let projection ctx n i =
  fn
    (Printf.sprintf "proj_%d_%d" n i)
    [
      block "initial" [ call ctx (aux_projection_init ctx n i) ];
      block "loop2_init" [ Lt; Break (head_is b); Continue None ];
      block "check_left" [ Lt; Return (head_is x); Rt ];
      block "initial_B" [ call_r ctx 1; put b; store "put" (Const x) ];
      block "shift_to_left_until_B"
        ([ Lt ] @ swap @ [ Break (var_is "put" b); Continue None ]);
      block "finally_B" [ Lt; Jump ("check_left", None) ];
    ]

let composition ctx inner outer =
  let n = List.length inner in
  if n = 0 then failwith "Empty composition";
  let rot = call ctx (rotate ctx n) in
  let blocks =
    [ block "initially" [ copy ctx (n - 1) ] ]
    @ List.mapi
        (fun i f ->
          block
            (Printf.sprintf "call_%d_%s" i f.name)
            ([ call_r ctx (n - 1); call ctx f; call_l ctx (n - 1) ]
            @ if i < n - 1 then [ rot ] else []))
        inner
  in
  (* Join the n scalar result tuples before invoking the outer function. *)
  let glue = call ctx (concat ctx) in
  let blocks =
    blocks
    @ List.init (n - 1) (fun i -> block (Printf.sprintf "concat_%d" i) [ glue ])
    @ [ block "outer" [ call ctx outer ] ]
  in
  fn
    ("comp_" ^ outer.name ^ "_"
    ^ String.concat "_" (List.map (fun f -> f.name) inner))
    blocks

(* Shift the argument tuple two cells to insert the x,l,x marker without
   losing either pending symbol, even when the first argument is zero. *)
let insert_sig ctx =
  fn "insert_sig"
    [
      block "initially" [ store "tmp1" (Const l); store "tmp2" (Const x); Rt ];
      block "main_loop"
        [
          Break (head_is x);
          store "tmp3" Head;
          put_var "tmp1";
          store "tmp1" (Var "tmp2");
          store "tmp2" (Var "tmp3");
          Rt;
          Continue None;
        ];
      block "finally"
        [ put_var "tmp1"; Rt; put_var "tmp2"; Rt; put x; call_l ctx 1 ];
    ]

let pred_tuple ctx =
  fn "pred_tuple"
    [
      block "initially"
        [
          call_r ctx 1;
          call ctx (shift_left_x ctx b);
          put x;
          Rt;
          Break (head_is x);
          put b;
        ];
      block "finally" [ call_l ctx 1 ];
    ]

let expand_arg ctx =
  let pred = call ctx (pred_tuple ctx) and cp = call ctx (copy_n_times ctx 1) in
  fn "expand_arg"
    [
      block "initially" [ call ctx (insert_sig ctx) ];
      block "check_non_zero"
        [
          Rt;
          Rt;
          Jump ("pred_if_non_zero", head_is l);
          Jump ("pred_if_zero", None);
        ];
      block "pred_if_non_zero"
        [ Lt; Lt; pred; cp; call_r ctx 1; Jump ("check_non_zero", None) ];
      block "pred_if_zero" [ Lt; Lt; pred ];
    ]

(* Expand descending arguments to the right, then fold results to the left.
   The x,l,x marker is outside the natural-number tuple alphabet. *)
let primitive_recursion ctx zero succ =
  let sw = call ctx (swap_tuple ctx)
  and glue = call ctx (concat ctx)
  and shift = call ctx (shift_left_x ctx b) in
  fn
    ("prim_" ^ zero.name ^ "_" ^ succ.name)
    [
      block "call_zero" [ call ctx (expand_arg ctx); call ctx zero ];
      block "loop"
        [
          call_l ctx 1;
          Rt;
          Break (head_is l);
          Lt;
          sw;
          glue;
          call ctx succ;
          Continue None;
        ];
      block "finally"
        [ Rt; put b; call_r ctx 1; shift; put x; call_r ctx 1; shift; put x ];
    ]

let delete ctx =
  fn "delete"
    [
      block "loop" [ Rt; Break (head_is x); put b; Continue None ];
      block "finally" [ put b; call_l ctx 1 ];
    ]

let mu_recursion ctx f =
  let c0 = call ctx (copy_to_end ctx 0) and c2 = call ctx (copy_to_end ctx 2) in
  let glue = call ctx (concat ctx)
  and sw = call ctx (swap_tuple ctx)
  and del = call ctx (delete ctx) in
  let fcall = call ctx f in
  fn ("mu_" ^ f.name)
    [
      block "initially" [ call_r ctx 1; Rt; put b; Rt; put x; call_l ctx 1 ];
      block "generate_and_call"
        [
          c0;
          call_l ctx 1;
          c2;
          call_r ctx 2;
          glue;
          fcall;
          Rt;
          Rt;
          Jump ("is_zero", head_is x);
          call_r ctx 1;
        ];
      block "is_non_zero" [ put b; Lt; Break (head_is x); Continue None ];
      block "next_num"
        [ put l; Rt; put x; call_l ctx 1; Jump ("generate_and_call", None) ];
      block "is_zero"
        [ put b; call_l ctx 3; sw; call_r ctx 1; del; call_l ctx 1 ];
    ]

let wrap_function ctx f =
  let main = fn "main" [ block "main" [ call ctx f ] ] in
  let program = { I.alphabet; functions = main :: ctx.functions } in
  { program with functions = I.dependency_order program }

let rec compile_function ctx = function
  | R.Zero -> zero_function ()
  | R.Succ -> succ_function ctx
  | R.Proj (n, i) -> projection ctx n i
  | R.Comp (outer, inners) ->
      let outer = compile_function ctx outer in
      let inners = List.map (compile_function ctx) inners in
      composition ctx inners outer
  | R.Prim (z, s) ->
      let z = compile_function ctx z in
      let s = compile_function ctx s in
      primitive_recursion ctx z s
  | R.Mu f -> mu_recursion ctx (compile_function ctx f)

let compile f =
  let* () = R.validate f in
  L.protect (fun () ->
      let ctx = create () in
      wrap_function ctx (compile_function ctx f))

module Source = R
module Target = I

let encode_ainput = write
let encode_rinput () = Ok ()
let decode_routput () = Ok ()

let decode_foutput t =
  match read_right_one t with
  | Some [ n ] -> Ok n
  | Some xs ->
      Error
        (Printf.sprintf
           "expected a single output value, but got tuple of length %d"
           (List.length xs))
  | None -> Error "failed to decode tape as recursive_function output"

let api =
  Utils_runtime.Compiler_text_api.make
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
