open Utils_runtime.Machine

module Code = struct
  type t = int

  let validate rule =
    if rule >= 0 && rule <= 255 then Ok rule
    else Error "Expected an elementary cellular automaton rule (0..255)"

  let parse text = Result.bind (Utils_data.Text.Natural_int.parse text) validate
  let print = string_of_int
end

module Ainput = Cells.Row
module Rinput = Utils_data.Text.Unit
module Routput = Cells.Row
module Foutput = Utils_data.Text.Unit

type state = {
  rule : int;
  generation : int;
  background : bool;
  cells : bool list;
}
[@@deriving yojson]

let make rule cells =
  let* rule = Code.validate rule in
  if cells = [] then Error "Expected a nonempty row"
  else Ok { rule; generation = 0; background = false; cells }

let transition rule left center right =
  let index =
    (Bool.to_int left * 4) + (Bool.to_int center * 2) + Bool.to_int right
  in
  rule land (1 lsl index) <> 0

let step s () =
  if s.generation = max_int then Error "Generation overflow"
  else
    let cells = Array.of_list s.cells in
    let at i =
      if i < 0 || i >= Array.length cells then s.background else cells.(i)
    in
    (* Extend by one cell on each side; all other cells remain uniform. *)
    let cells =
      List.init
        (Array.length cells + 2)
        (fun i -> transition s.rule (at (i - 2)) (at (i - 1)) (at i))
    in
    let background = transition s.rule s.background s.background s.background in
    Ok
      (Continue
         {
           next = { s with generation = s.generation + 1; background; cells };
           output = cells;
         })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    let* _ = Code.validate s.rule in
    if s.cells = [] then Error "Expected a nonempty row"
    else if s.generation < 0 || s.generation > (List.length s.cells - 1) / 2
    then Error "Invalid generation for the stored row"
    else Ok s
end

let render s =
  let open Utils_runtime.Render in
  [
    text ~title:"rule" (Code.print s.rule);
    text ~title:"generation" (string_of_int s.generation);
    text ~title:"left coordinate" (string_of_int (-s.generation));
    text ~title:"background" (Cells.Row.print [ s.background ]);
    table ~title:"cells" [ Cells.Row.render s.cells ];
  ]
