type direction = Right | Constant | Left [@@deriving yojson]

type 'a t = { left : 'a list; head : 'a; right : 'a list } [@@deriving yojson]
(** Both sides are stored nearest to the head first. Cells beyond the stored
    sides contain the default supplied to the operations below. *)

val blank : default:'a -> 'a t

val from_vec : default:'a -> 'a list -> int -> ('a t, string) result
(** The head may be just after the supplied cells, on a default cell. *)

val into_vec : 'a t -> 'a list * int
val head_write : 'a t -> 'a -> 'a t
val move_to : default:'a -> 'a t -> direction -> 'a t

val equal : default:'a -> equal:('a -> 'a -> bool) -> 'a t -> 'a t -> bool
(** Ignores trailing default cells on either side. *)
