type t
(** Immutable instruction sequences with precomputed matching loop brackets. *)

val create :
  opening:string ->
  closing:string ->
  allow_empty:bool ->
  string list ->
  (t, string) result

val length : t -> int
val token : t -> int -> string
val matching : t -> int -> int
val tokens : t -> string list
val print : t -> string
