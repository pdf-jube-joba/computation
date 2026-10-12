module type S = sig
  type t

  val parse : string -> (t, string) result
  val print : t -> string
end

module Unit = struct
  type t = unit

  let parse text =
    if String.trim text = "" then Ok () else Error "Expected empty input"

  let print () = ""
end

module String = struct
  type t = string

  let parse text = Ok text
  let print text = text
end

module Natural = struct
  type t = Number.t

  let parse = Number.of_string_with_plus
  let print = Number.to_string
end

module Natural_int = struct
  type t = int

  let parse text =
    match int_of_string_opt (Stdlib.String.trim text) with
    | Some n when n >= 0 -> Ok n
    | _ -> Error "Expected a nonnegative OCaml integer"

  let print = string_of_int
end

module Natural_list = struct
  type t = Number.t list

  let parse =
    Parser.parse (fun p ->
        Parser.group p ~opening:'(' (fun p ->
            Parser.separated_list ~skip_empty:true p ~separator:','
              Parser.natural_with_plus))

  let print values =
    "(" ^ Stdlib.String.concat ", " (List.map Natural.print values) ^ ")"
end
