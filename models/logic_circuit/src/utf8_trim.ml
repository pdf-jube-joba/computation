(* str::trim uses Unicode White_Space; String.trim only covers a subset of its
   ASCII members. Pin identifiers themselves remain strictly ASCII. *)
let whitespace_length text i =
  let n = String.length text in
  let byte j = Char.code text.[j] in
  match byte i with
  | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d | 0x20 -> 1
  | 0xc2 when i + 1 < n && (byte (i + 1) = 0x85 || byte (i + 1) = 0xa0) -> 2
  | 0xe1 when i + 2 < n && byte (i + 1) = 0x9a && byte (i + 2) = 0x80 -> 3
  | 0xe2
    when i + 2 < n
         && byte (i + 1) = 0x80
         && ((byte (i + 2) >= 0x80 && byte (i + 2) <= 0x8a)
            || byte (i + 2) = 0xa8
            || byte (i + 2) = 0xa9
            || byte (i + 2) = 0xaf) ->
      3
  | 0xe2 when i + 2 < n && byte (i + 1) = 0x81 && byte (i + 2) = 0x9f -> 3
  | 0xe3 when i + 2 < n && byte (i + 1) = 0x80 && byte (i + 2) = 0x80 -> 3
  | _ -> 0

let trim text =
  let n = String.length text in
  let rec start i =
    if i >= n then n
    else
      match whitespace_length text i with 0 -> i | length -> start (i + length)
  in
  let first = start 0 in
  let rec finish i last =
    if i >= n then last
    else
      match whitespace_length text i with
      | 0 -> finish (i + 1) (i + 1)
      | length -> finish (i + length) last
  in
  let last = finish first first in
  String.sub text first (last - first)
