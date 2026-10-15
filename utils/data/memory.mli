type 'a t = 'a array [@@deriving yojson]

val read : default:'a -> 'a t -> int -> 'a
(** Cells beyond the stored array read as [default]. Negative addresses fail. *)

val write : default:'a -> 'a t -> int -> 'a -> 'a t
(** Returns a copy, extending with [default] cells as needed. Invalid addresses
    fail. *)
