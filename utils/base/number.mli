type t
(** Arbitrary-precision natural numbers in canonical decimal notation. *)

val zero : t
val one : t
val of_string : string -> (t, string) result
val of_string_with_plus : string -> (t, string) result
val of_int : int -> t
val to_string : t -> string
val to_int : t -> (int, string) result
val compare : t -> t -> int
val equal : t -> t -> bool
val is_zero : t -> bool
val add : t -> t -> t
val sub : t -> t -> t
val succ : t -> t
val pred : t -> t
val mul : t -> t -> t
val divmod : t -> t -> (t * t, string) result
val of_bytes : string -> t
val to_bytes : t -> string
val to_yojson : t -> Yojson.Safe.t
val of_yojson : Yojson.Safe.t -> (t, string) result
