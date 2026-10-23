module Row = struct
  type t = bool list

  let parse text =
    let text = String.trim text in
    if text = "" then Error "Expected a nonempty row of 0/1 or ./# cells"
    else
      let rec read i acc =
        if i = String.length text then Ok (List.rev acc)
        else
          match text.[i] with
          | '0' | '.' -> read (i + 1) (false :: acc)
          | '1' | '#' -> read (i + 1) (true :: acc)
          | _ -> Error "Expected only 0/1 or ./# cells in a row"
      in
      read 0 []

  let print cells =
    String.of_seq
      (List.to_seq (List.map (fun b -> if b then '1' else '0') cells))

  let render cells =
    List.map (fun b -> Utils_runtime.Render.text (if b then "■" else "□")) cells
end

module Grid = struct
  type t = Row.t list

  let validate = function
    | [] | [] :: _ -> Error "Expected a nonempty grid"
    | row :: rows as grid ->
        if List.for_all (fun r -> List.length r = List.length row) rows then
          Ok grid
        else Error "Grid rows must have the same width"

  let parse text =
    let rec read acc = function
      | [] -> validate (List.rev acc)
      | line :: lines ->
          Result.bind (Row.parse line) (fun row -> read (row :: acc) lines)
    in
    read [] (String.split_on_char '\n' (String.trim text))

  let print grid = String.concat "\n" (List.map Row.print grid)
end
