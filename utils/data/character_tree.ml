type t = AtomValue of string | ConsValue of t * t [@@deriving yojson]

module P = Utils_parsing.Parser

let rec read_character p =
  if P.is_group p P.Parentheses then P.group p P.Parentheses read_character
  else P.character_literal p

let rec read p =
  if P.is_group p P.Parentheses then P.group p P.Parentheses read
  else if P.bump_if_keyword p "atom" then AtomValue (read_character p)
  else (
    P.expect_keyword p "cons";
    let left = read p in
    let right = read p in
    ConsValue (left, right))

let parse = P.parse ~character_literals:true read

let rec print = function
  | AtomValue c -> "atom " ^ Utils_parsing.Char_literal.to_text c
  | ConsValue (a, b) -> "cons " ^ print a ^ " " ^ print b
