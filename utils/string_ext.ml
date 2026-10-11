let is_whitespace = function ' ' | '\t' | '\r' | '\n' -> true | _ -> false

let starts_at text i prefix =
  i + String.length prefix <= String.length text
  && String.sub text i (String.length prefix) = prefix

let skip_while predicate text i =
  let j = ref i in
  while !j < String.length text && predicate text.[!j] do
    incr j
  done;
  !j

let skip_line text i = skip_while (( <> ) '\n') text i

let strip_prefix prefix text =
  if String.starts_with ~prefix text then
    String.sub text (String.length prefix)
      (String.length text - String.length prefix)
  else text

let strip_suffix suffix text =
  if String.ends_with ~suffix text then
    String.sub text 0 (String.length text - String.length suffix)
  else text

let split_once separator text =
  let len = String.length separator in
  let rec find i =
    if i + len > String.length text then None
    else if starts_at text i separator then
      Some
        ( String.sub text 0 i,
          String.sub text (i + len) (String.length text - i - len) )
    else find (i + 1)
  in
  find 0

let strip_line_comment marker line =
  match split_once marker line with Some (text, _) -> text | None -> line

let split_nonempty separator text =
  String.split_on_char separator text
  |> List.map String.trim
  |> List.filter (( <> ) "")

let lines text = split_nonempty '\n' text

let words text =
  String.map (fun c -> if is_whitespace c then ' ' else c) text
  |> String.split_on_char ' '
  |> List.filter (( <> ) "")
