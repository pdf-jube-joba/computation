(** Conversion of expected operational exceptions at result-returning
    boundaries. *)
let ( let* ) = Result.bind

let get = function Ok value -> value | Error message -> failwith message

let protect f =
  try Ok (f ())
  with Failure message | Invalid_argument message -> Error message
