open Utils.Machine
module Source = Recursive_function
module Target = Lambda_calculus

let v var = Target.Var var
let abs var body = Target.Abs (var, body)
let app left right = Target.App (left, right)
let abs_vs vars term = List.fold_right abs vars term
let apps first rest = List.fold_left app first rest

let fold_left = function
  | first :: rest -> apps first rest
  | [] -> invalid_arg "An application needs at least one term"

let if_lambda condition yes no = apps condition [ yes; no ]

(* One allocator per construction. Every binder is distinct, even when names
   coincide in nested compiled functions. Imported terms keep their free IDs. *)
let fresh next name =
  let id = !next in
  incr next;
  { Target.id; name }

let vars next prefix count =
  List.init count (fun i -> fresh next (prefix ^ string_of_int i))

let context terms =
  ref
    (List.fold_left
       (fun highest term -> max highest (Target.max_id term))
       (-1) terms
    + 1)

let church next number =
  let s = fresh next "s" in
  let z = fresh next "z" in
  let body = ref (v z) in
  for _ = 1 to number do
    body := app (v s) !body
  done;
  abs_vs [ s; z ] !body

let number_to_lambda_term number =
  let* number = Utils.Number.of_string (Utils.Number.to_string number) in
  let* number = Utils.Number.to_int number in
  Ok (church (ref 0) number)

let lambda_term_to_number = function
  | Target.Abs (s, Target.Abs (z, body)) when s.id <> z.id ->
      let rec count number = function
        | Target.Var var when var.id = z.id -> Some number
        | Target.App (Target.Var var, rest) when var.id = s.id ->
            count (Utils.Number.succ number) rest
        | _ -> None
      in
      count Utils.Number.zero body
  | _ -> None

let build_y next =
  let f = fresh next "x" in
  let x = fresh next "f" in
  let inner = abs x (app (v f) (app (v x) (v x))) in
  let second = Target.freshen_bound inner next in
  abs f (app inner second)

let build_true next =
  let x = fresh next "x" in
  let y = fresh next "y" in
  abs_vs [ x; y ] (v x)

let build_false next =
  let x = fresh next "x" in
  let y = fresh next "y" in
  abs_vs [ x; y ] (v y)

let build_succ next =
  let x = fresh next "x_succ" in
  let y = fresh next "y_succ" in
  let z = fresh next "z_succ" in
  abs_vs [ x; y; z ] (app (v y) (apps (v x) [ v y; v z ]))

let build_pred next =
  let x = fresh next "x_pred" in
  let y = fresh next "y_pred" in
  let z = fresh next "z_pred" in
  let p = fresh next "p_pred" in
  let q = fresh next "q_pred" in
  let ignored = fresh next "v_pred" in
  let identity = fresh next "v_pred" in
  let m = abs_vs [ p; q ] (app (v q) (app (v p) (v y))) in
  let n = abs ignored (v z) in
  let l = abs identity (v identity) in
  abs_vs [ x; y; z ] (apps (v x) [ m; n; l ])

let build_is_zero next =
  let n = fresh next "n" in
  let ignored = fresh next "_" in
  let falsity = build_false next in
  let truth = build_true next in
  abs n (apps (v n) [ abs ignored falsity; truth ])

let build_projection next n i =
  let variables = vars next "x" n in
  abs_vs variables (v (List.nth variables i))

let build_composition next n inner outer =
  let variables = vars next "x" n in
  let arguments = List.map v variables in
  abs_vs variables
    (apps outer (List.map (fun term -> apps term arguments) inner))

let build_primitive next n zero succ =
  let variables = vars next "x" n in
  let this = fresh next "THIS" in
  let first = v (List.hd variables) in
  let rest = List.map v (List.tl variables) in
  let zero_test = app (build_is_zero next) first in
  let base = apps zero rest in
  let predecessor = app (build_pred next) first in
  let recursive_call = apps (v this) (predecessor :: rest) in
  let predecessor_copy = Target.freshen_bound predecessor next in
  let successor = apps succ (recursive_call :: predecessor_copy :: rest) in
  let body = abs_vs (this :: variables) (if_lambda zero_test base successor) in
  app (build_y next) body

(* n is the arity AFTER minimization: the predicate receives n+1 arguments.
   Search tests f(index,args), starting at zero, before incrementing the index. *)
let build_mu next n predicate =
  let variables = vars next "v" (n + 1) in
  let inc = fresh next "INC" in
  let index = v (List.hd variables) in
  let rest = List.map v (List.tl variables) in
  let value = apps predicate (List.map v variables) in
  let zero_test = app (build_is_zero next) value in
  let next_index = app (build_succ next) index in
  let otherwise = apps (v inc) (next_index :: rest) in
  let body = abs_vs (inc :: variables) (if_lambda zero_test index otherwise) in
  let search = app (build_y next) body in
  let zero = church next 0 in
  abs_vs (List.tl variables) (apps search (zero :: rest))

let y_combinator () = build_y (ref 0)
let true_lambda () = build_true (ref 0)
let false_lambda () = build_false (ref 0)
let succ () = build_succ (ref 0)
let pred () = build_pred (ref 0)
let is_zero () = build_is_zero (ref 0)

let projection n i =
  if i < 0 || i >= n then None else Some (build_projection (ref 0) n i)

let composition n inner outer =
  let next = context (outer :: inner) in
  let outer = Target.freshen_bound outer next in
  let inner = List.map (fun term -> Target.freshen_bound term next) inner in
  build_composition next n inner outer

let primitive_recursion n zero succ =
  if n < 1 then invalid_arg "Primitive recursion requires a first argument";
  let next = context [ zero; succ ] in
  let zero = Target.freshen_bound zero next in
  let succ = Target.freshen_bound succ next in
  build_primitive next n zero succ

let mu_recursion n predicate =
  if n < 0 then invalid_arg "Mu arity must be nonnegative";
  let next = context [ predicate ] in
  let predicate = Target.freshen_bound predicate next in
  build_mu next n predicate

let rec compile_with next = function
  | Source.Zero -> church next 0
  | Source.Succ -> build_succ next
  | Source.Proj (n, i) -> build_projection next n i
  | Source.Comp (outer, inner) ->
      let outer_term = compile_with next outer in
      let inner_terms = List.map (compile_with next) inner in
      build_composition next
        (Source.arity (List.hd inner))
        inner_terms outer_term
  | Source.Prim (zero, succ) ->
      let zero_term = compile_with next zero in
      let succ_term = compile_with next succ in
      build_primitive next (Source.arity zero + 1) zero_term succ_term
  | Source.Mu predicate ->
      let predicate_term = compile_with next predicate in
      build_mu next (Source.arity predicate - 1) predicate_term

let compile func =
  let* () = Source.validate func in
  Ok (compile_with (ref 0) func)

let encode_ainput input =
  let rec encode = function
    | [] -> Ok []
    | number :: rest ->
        let* term = number_to_lambda_term number in
        let* rest = encode rest in
        Ok (term :: rest)
  in
  encode input

let encode_rinput () = Ok 0
let decode_routput () = Ok ()

let decode_foutput term =
  match lambda_term_to_number term with
  | Some number -> Ok number
  | None -> Error ("failed to decode Church numeral: " ^ Target.print term)

let api : Utils.Compiler.api =
  {
    compile_code =
      (fun text ->
        let* func = Source.parse_code text in
        let* term = compile func in
        Ok (Target.print term));
    encode_ainput =
      (fun text ->
        let* input = Source.parse_ainput text in
        let* terms = encode_ainput input in
        Ok (String.concat ", " (List.map Target.print terms)));
    encode_rinput =
      (fun text ->
        let* () = Source.parse_rinput text in
        let* index = encode_rinput () in
        Ok (string_of_int index));
    decode_routput =
      (fun text ->
        let* () = Utils.Machine.empty_input text in
        let* () = decode_routput () in
        Ok "");
    decode_foutput =
      (fun text ->
        let* term = Target.parse_code text in
        let* number = decode_foutput term in
        Ok (Utils.Number.to_string number));
  }

module Rec2LamCompiler = struct
  module Source = Source
  module Target = Target

  let compile = compile
  let encode_ainput = encode_ainput
  let encode_rinput = encode_rinput
  let decode_routput = decode_routput
  let decode_foutput = decode_foutput
  let api = api
end

module Compiler = Rec2LamCompiler
