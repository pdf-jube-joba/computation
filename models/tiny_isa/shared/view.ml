let registers regs =
  Utils_runtime.Render.indexed_table ~title:"registers"
    ~columns:[ "reg"; "value" ]
    ~index_text:(fun i -> "r" ^ string_of_int i)
    Utils_base.Number.to_string (Array.to_list regs)
