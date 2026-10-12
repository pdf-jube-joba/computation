let set key value entries =
  if List.mem_assoc key entries then
    List.map (fun (k, v) -> (k, if k = key then value else v)) entries
  else entries @ [ (key, value) ]

let sorted entries = List.sort (fun (a, _) (b, _) -> compare a b) entries

let set_sorted key value entries =
  sorted ((key, value) :: List.remove_assoc key entries)

let lookup key entries error =
  match List.assoc_opt key entries with
  | Some value -> value
  | None -> failwith error
