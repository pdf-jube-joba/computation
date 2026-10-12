(** Keys with [compare = 0] share an ID. Model-specific normalization belongs in
    the caller; the table owns a local supply, with no process-global state. *)
module Make
    (Key : Map.OrderedType)
    (Id : sig
      type t

      module Supply : sig
        type id = t
        type t

        val empty : t
        val fresh : t -> id * t
      end
    end) : sig
  type t

  val empty : t
  val find : Key.t -> t -> Id.t option
  val intern : Key.t -> t -> Id.t * t
  val take : Key.t -> t ref -> Id.t
end
