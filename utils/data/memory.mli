val read : default:'a -> 'a array -> int -> 'a
(** Cells beyond the stored array read as [default]. Negative addresses fail. *)

val write : default:'a -> 'a array -> int -> 'a -> 'a array
(** Returns a copy, extending with [default] cells as needed. Invalid addresses
    fail. *)
