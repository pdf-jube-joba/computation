type step = Continued of string | Halted of string

module Make (_ : Machine.S) : sig
  type t

  val create : unit -> t
  val make : t -> string -> string -> (unit, string) result
  val step : t -> string -> (step, string) result
  val snapshot : t -> (Yojson.Safe.t, string) result
  val restore : t -> Yojson.Safe.t -> (unit, string) result
  val render : t -> (Yojson.Safe.t, string) result
end
