let get = function Ok value -> value | Error message -> failwith message
let error = function Error _ -> () | Ok _ -> failwith "Expected an error"
let parse = Lambda_calculus.Code.parse

let () =
  let module L = Lambda_calculus in
  let term = get (parse "(\\x. x) (\\y. y)") in
  assert (L.Code.print (get (L.reduce term 0)) = "\\y. y");
  assert (L.redex_count (get (parse "x y z")) = 0);
  assert (L.Code.print (get (parse "x y z")) = "((x y) z)");
  assert (L.Code.print (get (parse "\\x y. x")) = "\\x. \\y. x");
  assert (L.Code.print (get (parse "/*comment*/ \\x. x // end")) = "\\x. x");
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
  let printed = get (parse (L.Code.print reduced)) in
  (match printed with
  | L.Abs (bound, L.Var free) -> assert (bound.id <> free.id)
  | _ -> assert false);
  let shadow = get (L.reduce (get (parse "(\\x. \\x. x) y")) 0) in
  (match shadow with
  | L.Abs (bound, L.Var var) -> assert (bound.id = var.id)
  | _ -> assert false);
  let multi = get (parse "((\\x. x) a) ((\\y. y) b)") in
  assert (L.Code.print (get (L.reduce multi 1)) = "(((\\x. x) a) b)");
  error (L.reduce multi 2);
  let roundtrip = get (L.restore (L.snapshot multi)) in
  assert (roundtrip = multi);
  assert (get (L.reduce roundtrip 0) = get (L.reduce multi 0));
  let free_identity = get (parse "(\\x. x y) z") in
  (match (free_identity, get (L.reduce free_identity 0)) with
  | L.App (L.Abs (_, L.App (_, L.Var y)), _), L.App (_, L.Var y') ->
      assert (y.id = y'.id)
  | _ -> assert false);
  assert (L.redex_count (get (parse (L.Code.print multi))) = L.redex_count multi);
  let applied =
    get (L.make (get (parse "\\x. \\y. x")) (get (L.Ainput.parse "y")))
  in
  (match get (L.reduce applied 0) with
  | L.Abs (bound, L.Var free) -> assert (bound.id <> free.id)
  | _ -> assert false);
  (* Repeated beta reduction must not capture IDs in shared argument subtrees. *)
  let shared = get (parse "(\\x. x x) (\\y. \\z. y)") in
  let twice = get (L.reduce (get (L.reduce shared 0)) 0) in
  assert (L.redex_count twice = 0);
  assert (get (L.restore (L.snapshot twice)) = twice);
  print_endline "Lambda calculus tests passed"
