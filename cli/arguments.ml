let rec options = function
  | [] -> []
  | key :: value :: rest when String.starts_with ~prefix:"--" key ->
      (key, value) :: options rest
  | args -> failwith ("Unexpected arguments: " ^ String.concat " " args)

let opt fields key default = Utils_base.Assoc.get ~default key fields

let require fields key =
  match List.assoc_opt key fields with
  | Some value -> value
  | None -> failwith ("Missing " ^ key)
