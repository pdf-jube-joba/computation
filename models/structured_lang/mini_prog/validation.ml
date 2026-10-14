open Runtime_types

let validate (code : code) =
  Utils_names.Identifier.check_unique
    ~message:(fun n ->
      "duplicate function: " ^ Utils_names.Identifier.to_string n)
    (List.map (fun (f : fn_decl) -> f.name) code.functions)
