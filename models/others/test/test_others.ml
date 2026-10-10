let ok = function Ok x -> x | Error e -> failwith e

let () =
  assert (Others.add 2L 2L = 4L);
  assert (Others.add 0L Int64.max_int = Int64.max_int);
  assert (
    Others.uint64_to_string (Others.add Int64.max_int 1L)
    = "9223372036854775808");
  let largest = ok (Others.uint64_of_string "18446744073709551615") in
  assert (
    largest = -1L && Others.uint64_to_string largest = "18446744073709551615");
  assert (Others.add largest 1L = 0L);
  assert (Others.add_checked 2L 2L = Ok 4L);
  assert (Others.add_checked largest 1L = Error "u64 addition overflow");
  (match Others.uint64_of_string "18446744073709551616" with
  | Error _ -> ()
  | Ok _ -> assert false);
  print_endline "others tests passed"
