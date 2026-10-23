module P = Utils_parsing.Parser

type 'a t = 'a list [@@deriving yojson]

module Make (Value : Text.S) = struct
  type nonrec t = Value.t t

  let rec read p =
    if P.is_group p P.Parentheses then P.group p P.Parentheses read
    else
      P.group p P.Brackets (fun p ->
          P.separated_list p ~separator:',' (fun p ->
              let text = P.take_until p ~stop:(( = ) (P.Symbol ',')) in
              Utils_base.Error.get (Value.parse text)))

  let parse = P.parse ~character_literals:true read

  let print values =
    "[" ^ String.concat ", " (List.map Value.print values) ^ "]"
end

module Character = Make (Text.Char)
