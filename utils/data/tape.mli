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

module type VALUE = sig
  type t [@@deriving yojson]

  val default : t
  val equal : t -> t -> bool
  val parse : string -> (t, string) result
  val print : t -> string
end

module Direction : sig
  type t = direction = Right | Constant | Left

  include Text.S with type t := t
end

module Make (Value : VALUE) : sig
  module Sign : VALUE with type t = Value.t
  module Direction = Direction

  type nonrec direction = direction = Right | Constant | Left

  val direction_to_yojson : direction -> Yojson.Safe.t
  val direction_of_yojson : Yojson.Safe.t -> (direction, string) result

  type sign = Value.t [@@deriving yojson]
  type nonrec t = sign t

  val to_yojson : t -> Yojson.Safe.t
  val of_yojson : Yojson.Safe.t -> (t, string) result
  val blank : t
  val from_vec : sign list -> int -> (t, string) result
  val into_vec : t -> sign list * int
  val head_write : t -> sign -> t
  val move_to : t -> direction -> t
  val equal : t -> t -> bool

  include Text.S with type t := t
end

module Identifier : sig
  type sign = Utils_names.Identifier.t option [@@deriving yojson]
  type nonrec direction = direction = Right | Constant | Left

  val direction_to_yojson : direction -> Yojson.Safe.t
  val direction_of_yojson : Yojson.Safe.t -> (direction, string) result

  type nonrec t = sign t

  val to_yojson : t -> Yojson.Safe.t
  val of_yojson : Yojson.Safe.t -> (t, string) result

  module Sign : VALUE with type t = sign
  module Direction = Direction

  val blank : t
  val from_vec : sign list -> int -> (t, string) result
  val into_vec : t -> sign list * int
  val head_write : t -> sign -> t
  val move_to : t -> direction -> t
  val equal : t -> t -> bool

  include Text.S with type t := t
end
