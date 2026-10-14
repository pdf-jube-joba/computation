let validate regs =
  if Array.length regs <> 8 then failwith "Expected 8 registers"
