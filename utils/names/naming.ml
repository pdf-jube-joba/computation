module Make (Key : Map.OrderedType) = struct
  module Names = Map.Make (Key)

  type key = Key.t

  type t = {
    names : Identifier.t Names.t;
    used : Identifier.Set.t;
    next_suffix : int Identifier.Map.t;
  }

  let empty =
    {
      names = Names.empty;
      used = Identifier.Set.empty;
      next_suffix = Identifier.Map.empty;
    }

  let reserve name t = { t with used = Identifier.Set.add name t.used }
  let find key t = Names.find_opt key t.names

  let assign ~hint key t =
    match find key t with
    | Some name -> (name, t)
    | None ->
        if not (Identifier.valid hint) then invalid_arg "Invalid name hint";
        let name, next_suffix =
          if Identifier.Set.mem hint t.used then
            let start =
              Option.value ~default:0
                (Identifier.Map.find_opt hint t.next_suffix)
            in
            let rec choose i =
              let name = hint ^ "_" ^ string_of_int i in
              if Identifier.Set.mem name t.used then choose (i + 1)
              else (name, i + 1)
            in
            let name, next = choose start in
            (name, Identifier.Map.add hint next t.next_suffix)
          else (hint, t.next_suffix)
        in
        ( name,
          {
            names = Names.add key name t.names;
            used = Identifier.Set.add name t.used;
            next_suffix;
          } )
end
