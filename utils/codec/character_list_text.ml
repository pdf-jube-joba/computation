module P = Utils_parsing.Parser

type t = string list

let rec read p =
  if P.is_group p P.Parentheses then P.group p P.Parentheses read
  else
    P.group p P.Brackets (fun p ->
        P.separated_list p ~separator:',' P.character_literal)

let parse = P.parse ~character_literals:true read

let print cs =
  "["
  ^ String.concat ", " (List.map Utils_parsing.Char_literal.to_text cs)
  ^ "]"
