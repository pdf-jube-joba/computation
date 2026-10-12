module Checked_no_ext : Utils.Machine.S = Lambda_continuations_no_ext.Main
module Checked_ctrl : Utils.Machine.S = Lambda_continuations_ctrl.Main
module Checked_root : Utils.Machine.S = Lambda_continuations_no_ext.Main
module T = Lambda_continuations_shared.Syntax
module M = Lambda_continuations_no_ext.Main

let get = function Ok x -> x | Error e -> failwith e
let error = function Error _ -> () | Ok _ -> failwith "expected an error"
let parsed text = get (T.Code.parse text)

let rec eval fuel e =
  if fuel = 0 then failwith "did not halt"
  else match M.reduce e with None -> e | Some e -> eval (fuel - 1) e

let run text inputs =
  let code = parsed text in
  let rec loop fuel s =
    if fuel = 0 then failwith "machine did not halt";
    let s =
      get
        (M.restore
           (Yojson.Safe.from_string (Yojson.Safe.to_string (M.snapshot s))))
    in
    match get (M.step s ()) with
    | Utils.Machine.Halt v -> v
    | Utils.Machine.Continue { next; _ } -> loop (fuel - 1) next
  in
  loop 2000 (get (M.make code (get (M.Ainput.parse inputs))))

(* Bound variables are compared by lexical depth, never by a printed dummy name. *)
let alpha_eq a b =
  let open T in
  let rec index x i = function
    | [] -> None
    | y :: ys -> if x = y then Some i else index x (i + 1) ys
  in
  let rec eq left right a b =
    match (a, b) with
    | Var x, Var y -> (
        match (index x 0 left, index y 0 right) with
        | None, None -> x = y
        | i, j -> i = j)
    | Lam (x, a), Lam (y, b) -> eq (x :: left) (y :: right) a b
    | App (a, b), App (c, d) -> eq left right a c && eq left right b d
    | Zero, Zero -> true
    | Succ a, Succ b | Pred a, Pred b | Abort a, Abort b | Control a, Control b
      ->
        eq left right a b
    | IfZ (a, b, c), IfZ (d, e, f) ->
        eq left right a d && eq left right b e && eq left right c f
    | Let (x, a, b), Let (y, c, d) ->
        eq left right a c && eq (x :: left) (y :: right) b d
    | Rec (f, x, b), Rec (g, y, c) -> eq (x :: f :: left) (y :: g :: right) b c
    | _ -> false
  in
  eq [] [] a b

let () =
  (* Free/bound-variable and capture-avoiding substitution tests. *)
  let e = T.Var "x" in
  assert (T.Names.elements (T.free_variables e) = [ "x" ]);
  assert (T.Names.is_empty (T.bound_variables e));
  let e = T.Lam ("x", T.Var "x") in
  assert (T.Names.is_empty (T.free_variables e));
  assert (T.Names.elements (T.bound_variables e) = [ "x" ]);
  let e = T.App (T.Var "x", T.Lam ("x", T.Var "z")) in
  assert (T.Names.elements (T.free_variables e) = [ "x"; "z" ]);
  assert (T.Names.elements (T.bound_variables e) = [ "x" ]);
  let capture = T.subst (parsed "fun y => x") "x" (T.Var "y") in
  assert (T.Names.elements (T.free_variables capture) = [ "y" ]);
  assert (alpha_eq capture (parsed "fun z => y"));
  let shadow = parsed "fun x => x" in
  assert (alpha_eq (T.subst shadow "x" T.Zero) shadow);
  let e = T.subst (parsed "let y = x in y") "x" (T.Var "y") in
  assert (T.Names.elements (T.free_variables e) = [ "y" ]);
  let e = T.subst (parsed "rec f y = @(x y)") "x" (T.Var "y") in
  assert (T.Names.elements (T.free_variables e) = [ "y" ]);
  assert (M.is_value (T.Var "x") = None);
  assert (M.is_value (parsed "fun x => x") <> None);
  let term = parsed "@((fun x => x) (fun y => fun z => y))" in
  assert (M.is_value term = None);
  assert (alpha_eq (Option.get (M.reduce term)) (parsed "fun y => fun z => y"));
  assert (run "rec f f = f" "(1)" = T.Num "1");
  let double = "rec f x = ifz x then zero else succ succ @(f (pred x))" in
  assert (run double "(2)" = T.Num "4");
  assert (run "fun x => pred x" "(0)" = T.Num "0");
  assert (run "let x = succ zero in succ x" "()" = T.Num "2");
  assert (run "ifz zero then succ zero else bad" "()" = T.Num "1");
  assert (run "ifz succ zero then bad else zero" "()" = T.Num "0");
  let one = parsed "succ zero" and two = parsed "succ succ zero" in
  let app =
    T.App (T.App (T.Lam ("x", T.Var "x"), T.Lam ("y", T.Var "y")), T.Pred two)
  in
  assert (Option.get (M.reduce app) = T.App (T.Lam ("y", T.Var "y"), T.Pred two));
  assert (
    Option.get (M.reduce (T.App (T.Lam ("x", T.Var "x"), T.Pred two)))
    = T.App (T.Lam ("x", T.Var "x"), one));
  assert (eval 20 (T.Pred two) = one);
  assert (M.reduce (T.App (T.Zero, T.Zero)) = None);
  assert (M.reduce (T.Succ (T.Lam ("x", T.Var "x"))) = None);
  assert (M.reduce (T.Pred (T.Lam ("x", T.Var "x"))) = None);
  assert (M.reduce (T.IfZ (T.Lam ("x", T.Var "x"), T.Zero, T.Zero)) = None);
  List.iter
    (fun text -> error (M.step (get (M.make (parsed text) [])) ()))
    [ "x"; "@(zero zero)"; "succ (fun x => x)" ];
  assert (run "@((fun succ_x => succ_x) (succ zero))" "()" = T.Num "1");
  (* Ctrl's variable/value operations and state restoration. *)
  let ctrl =
    get
      (Lambda_continuations_ctrl.Main.Code.parse
         "control (fun k => abort @(k (succ zero)))")
  in
  assert (T.Names.is_empty (Lambda_continuations_ctrl.Main.free_variables ctrl));
  assert (
    Lambda_continuations_ctrl.Main.Names.elements
      (Lambda_continuations_ctrl.Main.bound_variables ctrl)
    = [ "k" ]);
  assert (
    Lambda_continuations_ctrl.Main.subst
      (Lambda_continuations_ctrl.Main.Abort
         (Lambda_continuations_ctrl.Main.Var "x"))
      "x" Lambda_continuations_ctrl.Main.Zero
    = Lambda_continuations_ctrl.Main.Abort Lambda_continuations_ctrl.Main.Zero);
  assert (
    Lambda_continuations_ctrl.Main.is_value Lambda_continuations_ctrl.Main.Zero
    = Some (Lambda_continuations_ctrl.Main.Num "0"));
  assert (
    get
      (Lambda_continuations_ctrl.Main.restore
         (Lambda_continuations_ctrl.Main.snapshot
            (get (Lambda_continuations_ctrl.Main.make ctrl []))))
    = get (Lambda_continuations_ctrl.Main.make ctrl []));
  print_endline "Lambda continuation semantic tests passed"
