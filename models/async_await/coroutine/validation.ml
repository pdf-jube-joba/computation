open Runtime_types

let validate code =
  let names = List.map (fun f -> f.name) code in
  Utils_names.Identifier.check_unique
    ~message:(fun name ->
      "duplicate function: " ^ Utils_names.Identifier.to_string name)
    names;
  if not (List.mem (Utils_names.Identifier.make_exn "main") names) then
    failwith "missing main function"
