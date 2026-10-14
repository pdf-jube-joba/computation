(** Assign printable identifiers to identities. Hints do not determine identity.
    Reserve or assign names that must survive before assigning other names. *)
module Make (Key : Map.OrderedType) : sig
  type key = Key.t
  type t

  val empty : t
  val reserve : Identifier.t -> t -> t
  val find : key -> t -> Identifier.t option

  val assign : hint:Identifier.t -> key -> t -> Identifier.t * t
  (** Reuses the existing name for a key. Otherwise uses [hint] if available, or
      a fresh [hint_0], [hint_1], ... and records the chosen name. *)
end

(** A local supply for generated printable names, using the same naming policy.
*)
module Fresh () : sig
  type t

  val create : ?reserved:Identifier.t list -> unit -> t
  val take : t -> Identifier.t -> Identifier.t
end
