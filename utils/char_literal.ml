let to_text = function
  | "\n" -> "'\\n'"
  | "\r" -> "'\\r'"
  | "\t" -> "'\\t'"
  | "\\" -> "'\\\\'"
  | "'" -> "'\\''"
  | c -> "'" ^ c ^ "'"

let read text i =
  let n = String.length text in
  let c, j =
    if i + 1 >= n then failwith "Unterminated character literal"
    else if text.[i + 1] = '\\' then (
      if i + 2 >= n then failwith "Unterminated escape";
      let c, k = Utf8.scalar text (i + 2) in
      ((match c with "n" -> "\n" | "r" -> "\r" | "t" -> "\t" | c -> c), k))
    else Utf8.scalar text (i + 1)
  in
  if j >= n || text.[j] <> '\'' then failwith "Expected closing quote";
  (c, j + 1)
