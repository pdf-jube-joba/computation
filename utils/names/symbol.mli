(** A typed identity paired with a source/display hint. Equality ignores hints.
*)
module Make () : sig
  module Id : module type of Id.Make ()
  module Names : module type of Naming.Make (Id)

  type t = { id : Id.t; name : Identifier.t }

  val compare : t -> t -> int
  val equal : t -> t -> bool
  val fresh : Id.Supply.t ref -> Identifier.t -> t
  val with_id : Id.t -> t -> t
  val to_yojson : t -> Yojson.Safe.t
  val of_yojson : Yojson.Safe.t -> (t, string) result

  module Reader : sig
    type symbol = t
    type t

    val create : unit -> t
    val bind : t -> Identifier.t -> symbol

    val use : t -> (Identifier.t * symbol) list -> Identifier.t -> symbol
    (** Finds the nearest binder, or interns a free name within this reader. *)
  end
end
