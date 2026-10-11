(* An int64 carries the full u64 bit pattern. Int64.add has the unsigned
   wrapping behavior of the original function in a release Rust build. *)
type uint64 = int64

let add = Int64.add

let add_checked left right =
  let sum = add left right in
  if Int64.unsigned_compare sum left < 0 then Error "u64 addition overflow"
  else Ok sum

let uint64_of_string text =
  try Ok (Int64.of_string ("0u" ^ text))
  with Failure _ -> Error "Expected an unsigned 64-bit decimal integer"

let uint64_to_string value = Printf.sprintf "%Lu" value
