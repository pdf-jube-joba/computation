type t
(** A validated ASCII identifier. Raw source text enters through [make]. *)

val is_start : char -> bool
val is_continue : char -> bool
val valid : string -> bool
val make : string -> (t, string) result
val make_exn : string -> t
val to_string : t -> string
val compare : t -> t -> int
val equal : t -> t -> bool
val to_yojson : t -> Yojson.Safe.t
val of_yojson : Yojson.Safe.t -> (t, string) result

module Map : Map.S with type key = t
module Set : Set.S with type elt = t

val check_unique : message:(t -> string) -> t list -> unit
val fresh : ?prefix:string -> Set.t -> t
