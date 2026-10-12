open Utils.Machine
module _ : Utils.Compiler.S = Example_counter_to_example_counter.Main

let get = function Ok value -> value | Error message -> failwith message
let error = function Error _ -> () | Ok _ -> failwith "Expected an error"

let () =
  let module C = Example_counter.Main in
  error (C.step 0 C.Decrement);
  error (C.step max_int C.Increment);
  assert (C.step 9 C.Increment = Ok (Halt "End"));
  assert (C.step 2 C.Decrement = Ok (Continue { next = 1; output = "dec" }));
  assert (get (C.restore (C.snapshot 7)) = 7);
  print_endline "Example counter tests passed"
