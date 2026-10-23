open Utils_runtime.Machine
module Code = Utils_data.Text.Unit
module Ainput = Cells.Grid
module Rinput = Utils_data.Text.Unit
module Routput = Cells.Grid
module Foutput = Utils_data.Text.Unit

type state = { generation : int; cells : bool list list } [@@deriving yojson]

let make () cells =
  let* cells = Cells.Grid.validate cells in
  Ok { generation = 0; cells }

let step s () =
  if s.generation = max_int then Error "Generation overflow"
  else
    let grid = Array.of_list (List.map Array.of_list s.cells) in
    let height = Array.length grid and width = Array.length grid.(0) in
    let at x y = x >= 0 && y >= 0 && x < width && y < height && grid.(y).(x) in
    let cells =
      List.init (height + 2) (fun row ->
          List.init (width + 2) (fun col ->
              let x = col - 1 and y = row - 1 in
              let neighbours = ref 0 in
              for dy = -1 to 1 do
                for dx = -1 to 1 do
                  if (dx <> 0 || dy <> 0) && at (x + dx) (y + dy) then
                    incr neighbours
                done
              done;
              !neighbours = 3 || (!neighbours = 2 && at x y)))
    in
    Ok
      (Continue
         { next = { generation = s.generation + 1; cells }; output = cells })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    let* cells = Cells.Grid.validate s.cells in
    let width = List.length (List.hd cells) and height = List.length cells in
    if s.generation < 0 || s.generation > (min width height - 1) / 2 then
      Error "Invalid generation for the stored grid"
    else Ok s
end

let render s =
  let open Utils_runtime.Render in
  [
    text ~title:"rule" "B3/S23";
    text ~title:"generation" (string_of_int s.generation);
    text ~title:"top-left coordinate"
      (Printf.sprintf "(%d, %d)" (-s.generation) (-s.generation));
    table ~title:"cells" (List.map Cells.Row.render s.cells);
  ]
