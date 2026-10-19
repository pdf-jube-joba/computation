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

module Char = struct
  (* One Unicode scalar, represented as UTF-8. *)
  type t = string

  module P = Utils_parsing.Parser

  let rec read p =
    if P.is_group p P.Parentheses then P.group p P.Parentheses read
    else P.character_literal p

  let parse = P.parse ~character_literals:true read
  let print = Utils_parsing.Char_literal.to_text
end

module Bool = struct
  type t = bool

  let read p =
    match Utils_parsing.Parser.keyword p with
    | "true" -> true
    | "false" -> false
    | word -> Utils_parsing.Parser.error p ("Invalid boolean: \\" ^ word)

  let parse = Utils_parsing.Parser.parse read
  let print = function true -> "\\true" | false -> "\\false"
end

module Natural = struct
  type t = Utils_base.Number.t

  let parse = Utils_base.Number.of_string_with_plus
  let print = Utils_base.Number.to_string
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
  type t = Utils_base.Number.t list

  let parse =
    Utils_parsing.Parser.parse (fun p ->
        Utils_parsing.Parser.group p Utils_parsing.Parser.Parentheses (fun p ->
            Utils_parsing.Parser.separated_list ~skip_empty:true p
              ~separator:',' Utils_parsing.Parser.natural_with_plus))

  let print values =
    "(" ^ Stdlib.String.concat ", " (List.map Natural.print values) ^ ")"
end
