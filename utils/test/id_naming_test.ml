module Id = Utils.Id.Make ()
module Names = Utils.Naming.Make (Id)
module Intern = Utils.Intern.Make (Utils.Identifier) (Id)

let get = function Ok value -> value | Error e -> failwith e

let () =
  let first, supply = Id.Supply.fresh Id.Supply.empty in
  let second, supply = Id.Supply.fresh supply in
  assert (not (Id.equal first second));
  let imported = get (Id.of_int 42) in
  let supply = Id.Supply.reserve imported supply in
  let supply = get (Id.Supply.of_yojson (Id.Supply.to_yojson supply)) in
  let next, _ = Id.Supply.fresh supply in
  assert (Id.to_int next = 43);
  assert (Id.Supply.validate_used (Id.Set.singleton imported) supply = Ok ());
  assert (
    Result.is_error
      (Id.Supply.validate_used (Id.Set.singleton imported) Id.Supply.empty));
  let next, _ =
    Id.Supply.fresh (Id.Supply.of_used (Id.Set.of_list [ second; imported ]))
  in
  assert (Id.to_int next = 43);
  assert (get (Id.of_yojson (Id.to_yojson imported)) = imported);
  assert (Result.is_error (Id.of_int (-1)));
  assert (Result.is_error (Id.of_int max_int));
  let exhausted = get (Id.Supply.of_yojson (`Int max_int)) in
  assert (
    try
      ignore (Id.Supply.fresh exhausted);
      false
    with Invalid_argument _ -> true);
  let names = Names.reserve "x_0" Names.empty in
  let a, names = Names.assign ~hint:"x" first names in
  let b, names = Names.assign ~hint:"x" second names in
  let again, _ = Names.assign ~hint:"other" first names in
  assert (a = "x" && b = "x_1" && again = a);
  assert (Names.find imported names = None);
  let a, table = Intern.intern "same" Intern.empty in
  let again, table = Intern.intern "same" table in
  let b, _ = Intern.intern "other" table in
  assert (Id.equal a again && not (Id.equal a b));
  print_endline "ID allocation and printable-name tests passed"
