open Utils.Machine

let get = function Ok value -> value | Error message -> failwith message
let error = function Error _ -> () | Ok _ -> failwith "Expected an error"
let parse = Lambda_calculus.parse_code

let () =
  let module C = Example_counter in
  error (C.step 0 C.Decrement);
  error (C.step max_int C.Increment);
  assert (C.step 9 C.Increment = Ok (Halt "End"));
  assert (C.step 2 C.Decrement = Ok (Continue { next = 1; output = "dec" }));
  assert (get (C.restore (C.snapshot 7)) = 7);
  let module L = Lambda_calculus in
  let term = get (parse "(\\x. x) (\\y. y)") in
  assert (L.print (get (L.reduce term 0)) = "\\y. y");
  assert (L.redex_count (get (parse "x y z")) = 0);
  assert (L.print (get (parse "x y z")) = "((x y) z)");
  assert (L.print (get (parse "\\x y. x")) = "\\x. \\y. x");
  assert (L.print (get (parse "/*comment*/ \\x. x // end")) = "\\x. x");
  assert (L.alpha_eq (get (parse "\\x. \\x. x")) (get (parse "\\a. \\b. b")));
  assert (
    not (L.alpha_eq (get (parse "\\x. \\x. x")) (get (parse "\\a. \\b. a"))));
  List.iter
    (fun text -> error (parse text))
    [ ""; "x)"; "(x"; "\\. x"; "\\x."; "x @ y" ];
  (* Free y must remain free after reduction beneath a y binder. *)
  let capture = get (parse "(\\x. \\y. x) y") in
  let reduced = get (L.reduce capture 0) in
  (match reduced with
  | L.Abs (bound, L.Var free) -> assert (bound.id <> free.id)
  | _ -> assert false);
  let printed = get (parse (L.print reduced)) in
  (match printed with
  | L.Abs (bound, L.Var free) -> assert (bound.id <> free.id)
  | _ -> assert false);
  let shadow = get (L.reduce (get (parse "(\\x. \\x. x) y")) 0) in
  (match shadow with
  | L.Abs (bound, L.Var var) -> assert (bound.id = var.id)
  | _ -> assert false);
  let multi = get (parse "((\\x. x) a) ((\\y. y) b)") in
  assert (L.print (get (L.reduce multi 1)) = "(((\\x. x) a) b)");
  error (L.reduce multi 2);
  let roundtrip = get (L.restore (L.snapshot multi)) in
  assert (roundtrip = multi);
  assert (get (L.reduce roundtrip 0) = get (L.reduce multi 0));
  let free_identity = get (parse "(\\x. x y) z") in
  (match (free_identity, get (L.reduce free_identity 0)) with
  | L.App (L.Abs (_, L.App (_, L.Var y)), _), L.App (_, L.Var y') ->
      assert (y.id = y'.id)
  | _ -> assert false);
  assert (L.redex_count (get (parse (L.print multi))) = L.redex_count multi);
  let applied =
    get (L.make (get (parse "\\x. \\y. x")) (get (L.parse_ainput "y")))
  in
  (match get (L.reduce applied 0) with
  | L.Abs (bound, L.Var free) -> assert (bound.id <> free.id)
  | _ -> assert false);
  (* Repeated beta reduction must not capture IDs in shared argument subtrees. *)
  let shared = get (parse "(\\x. x x) (\\y. \\z. y)") in
  let twice = get (L.reduce (get (L.reduce shared 0)) 0) in
  assert (L.redex_count twice = 0);
  assert (get (L.restore (L.snapshot twice)) = twice);
  let module S = Utils.Session.Make (C) in
  let a = S.create () and b = S.create () in
  ignore (get (S.make a "3" ""));
  ignore (get (S.make b "8" ""));
  error (S.step a "oops");
  assert (get (S.snapshot a) = C.snapshot 3);
  ignore (get (S.step a "inc"));
  assert (get (S.snapshot b) = C.snapshot 8);
  error (S.restore a (`String "bad"));
  assert (get (S.snapshot a) = C.snapshot 4);
  print_endline "Model tests passed"
