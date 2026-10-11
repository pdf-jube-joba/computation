let get = function Ok x -> x | Error e -> failwith e

let run initial =
  let rec loop fuel state =
    if fuel = 0 then failwith "Recursive function step limit exceeded";
    match get (Recursive_function.step state ()) with
    | Utils.Machine.Halt output -> output
    | Utils.Machine.Continue { next; _ } -> loop (fuel - 1) next
  in
  loop 100000 initial

let () =
  let module R = Recursive_function in
  let addition =
    get (R.Code.parse "let add = PRIM[z:PROJ[1,0] s:COMP[SUCC:PROJ[3,0]]]. add")
  in
  assert (run (get (R.make addition [ "3"; "5" ])) = "8");
  assert (R.Code.parse (R.Code.print addition) = Ok addition);
  let mu = get (R.Code.parse "MUOP[PROJ[1,0]]") in
  assert (run (get (R.make mu [])) = "0");
  let p = get (R.make addition [ "2"; "3" ]) in
  assert (get (R.restore (R.snapshot p)) = p);
  assert (match R.Code.parse "PROJ[1,2]" with Error _ -> true | _ -> false);
  assert (
    match R.Code.parse "PRIM[z:ZERO s:SUCC]" with Error _ -> true | _ -> false);
  print_endline "Recursive function tests passed"
