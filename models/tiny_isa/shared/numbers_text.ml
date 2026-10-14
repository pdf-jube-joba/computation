module N = Utils_base.Number

let value = Utils_base.Error.get

type t = N.t list

let parse text =
  Utils_parsing.Parser.protect (fun () ->
      List.map
        (fun n -> value (N.of_string_with_plus n))
        (Utils_base.String_ext.lines text))

let print ns = String.concat "\n" (List.map N.to_string ns)
