let get = function Ok value -> value | Error message -> failwith message
let error = function Error _ -> () | Ok _ -> failwith "Expected an error"

let () =
  let module C = Example_counter in
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
  print_endline "Session tests passed"
