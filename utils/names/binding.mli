(** Adapter for named syntax. Each child's scope lists the indices of the
    binders visible to it, from outermost to innermost. The rebuild function
    receives new binder names and children in their original order. *)
type 'a view =
  | Variable of Identifier.t
  | Node of
      Identifier.t list
      * (int list * 'a) list
      * (Identifier.t list -> 'a list -> 'a)

val node :
  ?binders:Identifier.t list ->
  (int list * 'a) list ->
  (Identifier.t list -> 'a list -> 'a) ->
  'a view

module Make
    (Syntax : sig
      type t

      val view : t -> t view
      val variable : Identifier.t -> t
    end)
    () : sig
  val normalize : Syntax.t -> Syntax.t
  (** Resolves binding identities and chooses distinct printable binder names,
      preserving free names. *)

  val substitute : Syntax.t -> Identifier.t -> Syntax.t -> Syntax.t
  (** Substitutes free occurrences using resolved IDs, with capture avoided. An
      absent target leaves the original term unchanged. *)
end
