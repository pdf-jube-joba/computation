type 'a t = AtomValue of 'a | ConsValue of 'a t * 'a t [@@deriving yojson]

module P = Utils_parsing.Parser

module Make (Value : Text.S) = struct
  type nonrec t = Value.t t

  let rec read p =
    if P.is_group p P.Parentheses then P.group p P.Parentheses read
    else if P.bump_if_keyword p "atom" then
      let start = P.offset p in
      let text =
        P.take_until p ~stop:(function
          | P.Keyword ("atom" | "cons") -> true
          | P.Symbol '(' -> P.offset p <> start
          | _ -> false)
      in
      AtomValue (Utils_base.Error.get (Value.parse text))
    else (
      P.expect_keyword p "cons";
      let left = read p in
      let right = read p in
      ConsValue (left, right))

  let parse = P.parse ~character_literals:true read

  let rec print = function
    | AtomValue value -> "\\atom " ^ Value.print value
    | ConsValue (a, b) -> "\\cons " ^ print a ^ " " ^ print b
end

module Character = Make (Text.Char)
