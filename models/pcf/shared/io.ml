module Ainput = Utils_data.Text.Natural_list

module Routput = struct
  type t = Syntax.print_effect

  let parse text =
    if String.trim text = "" then Ok None
    else Result.map Option.some (Syntax.N.of_string_with_plus text)

  let print = function None -> "" | Some n -> Syntax.N.to_string n
end

module Foutput = Utils_data.Text.Natural
