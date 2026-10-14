open Types

let runtime_text s v =
  Input_text.print_value (Resolution.named_value s.resolution v)

let register_id s name = Resolution.id_of_name s.resolution.registers name
