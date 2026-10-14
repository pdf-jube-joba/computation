val ( let* ) : ('a, 'e) result -> ('a -> ('b, 'e) result) -> ('b, 'e) result
val get : ('a, string) result -> 'a
val protect : (unit -> 'a) -> ('a, string) result
