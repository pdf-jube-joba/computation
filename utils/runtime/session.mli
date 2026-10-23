type step = Continued of string | Halted of string

module Make (M : Machine.S) : sig
  type t

  val create : unit -> t
  val make : t -> string -> string -> (unit, string) result
  val step : t -> string -> (step, string) result
  val snapshot : t -> (M.Snapshot.t, string) result
  val restore : t -> M.Snapshot.t -> (unit, string) result
  val render : t -> (Render.t, string) result
end
