module Make () = struct
  type t = int

  let compare = Int.compare
  let equal = Int.equal
  let to_int id = id

  let of_int id =
    if id >= 0 && id < max_int then Ok id
    else Error "ID must be nonnegative and smaller than max_int"

  let to_yojson id = `Int id

  let of_yojson = function
    | `Int id -> of_int id
    | _ -> Error "Expected integer ID"

  module Map = Map.Make (Int)
  module Set = Set.Make (Int)

  module Supply = struct
    type id = t
    type t = int

    let empty = 0
    let reserve id next = max next (id + 1)
    let after id = reserve id empty
    let of_used used = Set.fold reserve used empty
    let to_int next = next

    let validate_used used next =
      if Set.for_all (fun id -> id < next) used then Ok ()
      else Error "ID supply would reuse an existing ID"

    let fresh next =
      if next = max_int then invalid_arg "ID supply exhausted";
      (next, next + 1)

    let take supply =
      let id, next = fresh !supply in
      supply := next;
      id

    let to_yojson next = `Int next

    let of_yojson = function
      | `Int next when next >= 0 -> Ok next
      | _ -> Error "Expected nonnegative ID supply"
  end
end
