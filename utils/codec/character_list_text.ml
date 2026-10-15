module P = Utils_parsing.Parser

type t = Utils_data.Character_list.t

let rec read p =
  if P.peek p = P.Symbol '(' then P.group p ~opening:'(' read
  else
    P.group p ~opening:'[' (fun p ->
        P.separated_list p ~separator:',' P.character_literal)

let parse = P.parse ~character_literals:true read

let print cs =
  "["
  ^ String.concat ", " (List.map Utils_parsing.Char_literal.to_text cs)
  ^ "]"
