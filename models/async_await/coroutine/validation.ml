open Runtime_types

let validate code =
  let names = List.map (fun f -> f.name) code in
  Utils_names.Identifier.check_unique
    ~message:(fun name -> "duplicate function: " ^ name)
    names;
  if not (List.mem "main" names) then failwith "missing main function"
