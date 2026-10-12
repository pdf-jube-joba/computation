(** Sharing is determined by [Key.compare = 0]; normalize keys in the caller
    when sharing should be up to a model-specific equivalence. *)
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
    end) =
struct
  module Entries = Map.Make (Key)

  type t = { entries : Id.t Entries.t; supply : Id.Supply.t }

  let empty = { entries = Entries.empty; supply = Id.Supply.empty }
  let find key t = Entries.find_opt key t.entries

  let intern key t =
    match find key t with
    | Some id -> (id, t)
    | None ->
        let id, supply = Id.Supply.fresh t.supply in
        (id, { entries = Entries.add key id t.entries; supply })

  let take key t =
    let id, next = intern key !t in
    t := next;
    id
end
