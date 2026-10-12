(** A resolved occurrence: identity is independent of the source/display hint.
*)
module Make () = struct
  module Id = Id.Make ()
  module Names = Naming.Make (Id)

  type t = { id : Id.t; name : Identifier.t } [@@deriving yojson]

  let compare a b = Id.compare a.id b.id
  let equal a b = Id.equal a.id b.id
  let fresh supply name = { id = Id.Supply.take supply; name }
  let with_id id var = { var with id }

  module Reader = struct
    type symbol = t

    type t = {
      supply : Id.Supply.t ref;
      mutable free : symbol Identifier.Map.t;
    }

    let create () =
      { supply = ref Id.Supply.empty; free = Identifier.Map.empty }

    let bind t name = fresh t.supply name

    let use t env name =
      match List.assoc_opt name env with
      | Some symbol -> symbol
      | None -> (
          match Identifier.Map.find_opt name t.free with
          | Some symbol -> symbol
          | None ->
              let symbol = bind t name in
              t.free <- Identifier.Map.add name symbol t.free;
              symbol)
  end
end
