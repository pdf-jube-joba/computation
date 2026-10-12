(** Integer identities, distinct for each application of [Make ()]. Uniqueness
    is local to a supply. Independent supplies may issue equal IDs; callers must
    share a supply or remap IDs when combining their results. *)
module Make () : sig
  type t

  val compare : t -> t -> int
  val equal : t -> t -> bool
  val to_int : t -> int
  val of_int : int -> (t, string) result
  val to_yojson : t -> Yojson.Safe.t
  val of_yojson : Yojson.Safe.t -> (t, string) result

  module Map : Map.S with type key = t
  module Set : Set.S with type elt = t

  module Supply : sig
    type id = t
    type t

    val empty : t
    val after : id -> t
    val reserve : id -> t -> t
    val of_used : Set.t -> t
    val to_int : t -> int

    val validate_used : Set.t -> t -> (unit, string) result
    (** Checks that future IDs cannot collide with restored IDs. *)

    val fresh : t -> id * t
    (** Raises [Invalid_argument] if the integer range is exhausted. *)

    val take : t ref -> id
    (** Mutable convenience for a parser or compiler's local supply. *)

    val to_yojson : t -> Yojson.Safe.t
    val of_yojson : Yojson.Safe.t -> (t, string) result
  end
end
