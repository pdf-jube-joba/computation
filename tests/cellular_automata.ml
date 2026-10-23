module Machine = Utils_runtime.Machine
module E = Cellular_automaton.Main
module L = Cellular_automaton.Life

let get = function Ok x -> x | Error e -> failwith e
let check message condition = if not condition then failwith message

module Run (M : Machine.S) = struct
  let start code input =
    get (M.make (get (M.Code.parse code)) (get (M.Ainput.parse input)))

  let step state =
    let saved = M.Snapshot.to_yojson state in
    let restored =
      get
        (M.Snapshot.of_yojson
           (Yojson.Safe.from_string (Yojson.Safe.to_string saved)))
    in
    check "snapshot roundtrip" (M.Snapshot.to_yojson restored = saved);
    check "render roundtrip" (M.render restored = M.render state);
    let result = get (M.step restored (get (M.Rinput.parse ""))) in
    check "step mutated saved state" (M.Snapshot.to_yojson restored = saved);
    check "restored transition differs"
      (result = get (M.step state (get (M.Rinput.parse ""))));
    match result with
    | Machine.Continue { next; output } ->
        let printed = M.Routput.print output in
        check "output codec roundtrip"
          (M.Routput.print (get (M.Routput.parse printed)) = printed);
        next
    | Machine.Halt _ -> failwith "Cellular automata must keep evolving"

  let rec advance n state =
    if n = 0 then state else advance (n - 1) (step state)
end

module Elementary = Run (E)
module Life = Run (L)

let replace key value = function
  | `Assoc fields ->
      `Assoc (List.map (fun (k, v) -> (k, if k = key then value else v)) fields)
  | _ -> failwith "Expected object"

let elementary () =
  let initial = Elementary.start "30" "1" in
  let expected = [ "111"; "11001"; "1101111"; "110010001" ] in
  ignore
    (List.fold_left
       (fun state row ->
         let next = Elementary.step state in
         check "Rule 30 pattern" (E.Routput.print next.cells = row);
         next)
       initial expected);
  check "Rule 110 pattern"
    (E.Routput.print (Elementary.advance 2 (Elementary.start "110" "1")).cells
    = "11100");
  (* Independently check the eight neighbourhoods of all 256 rules. *)
  for rule = 0 to 255 do
    for neighbourhood = 0 to 7 do
      let input =
        List.init 3 (fun i -> neighbourhood land (1 lsl (2 - i)) <> 0)
      in
      let next = Elementary.step (get (E.make rule input)) in
      check "rule bit ordering"
        (List.nth next.cells 2 = (rule / (1 lsl neighbourhood) mod 2 = 1))
    done
  done;
  let toggling = Elementary.start "1" "0" in
  let odd = Elementary.step toggling in
  let even = Elementary.step odd in
  check "infinite background becomes live"
    (odd.background && E.Routput.print odd.cells = "111");
  check "infinite background becomes dead"
    ((not even.background) && E.Routput.print even.cells = "00000");
  let live = Elementary.advance 2 (Elementary.start "255" "0") in
  check "infinite background stays live"
    (live.background && List.for_all Fun.id live.cells);
  let dead = Elementary.advance 3 (Elementary.start "0" "1") in
  check "dead pattern keeps advancing"
    (dead.generation = 3 && List.for_all not dead.cells);
  List.iter
    (fun text ->
      check "bad rule accepted" (Result.is_error (E.Code.parse text)))
    [ ""; "-1"; "256"; "30 extra" ];
  List.iter
    (fun text ->
      check "bad row accepted" (Result.is_error (E.Ainput.parse text)))
    [ ""; "012"; "1 0"; "1\n0" ];
  check "row aliases" (get (E.Ainput.parse ".#.\r\n") = [ false; true; false ]);
  check "invalid typed rule accepted" (Result.is_error (E.make 256 [ true ]));
  check "empty typed row accepted" (Result.is_error (E.make 30 []));
  let saved = E.Snapshot.to_yojson initial in
  List.iter
    (fun bad ->
      check "bad elementary snapshot accepted"
        (Result.is_error (E.Snapshot.of_yojson bad)))
    [
      `Null;
      replace "rule" (`Int 256) saved;
      replace "generation" (`Int (-1)) saved;
      replace "generation" (`Int 1) saved;
      replace "cells" (`List []) saved;
      replace "cells" (`List [ `Int 2 ]) saved;
      replace "background" (`Int 0) saved;
    ];
  check "unexpected runtime input accepted"
    (Result.is_error (E.Rinput.parse "step"))

let positions (state : L.state) =
  List.concat
    (List.mapi
       (fun y row ->
         List.filter_map
           (fun (x, alive) ->
             if alive then Some (x - state.generation, y - state.generation)
             else None)
           (List.mapi (fun x alive -> (x, alive)) row))
       state.cells)
  |> List.sort compare

let life () =
  let blinker = Life.start "" "###" in
  check "blinker grows outside input bounds"
    (positions (Life.step blinker) = [ (1, -1); (1, 0); (1, 1) ]);
  check "blinker period two"
    (positions (Life.advance 2 blinker) = positions blinker);
  let block = Life.start "" "##\n##" in
  check "block remains stable"
    (positions (Life.advance 4 block) = positions block);
  let glider = Life.start "" ".#.\n..#\n###" in
  check "glider moves diagonally after four generations"
    (positions (Life.advance 4 glider)
    = List.map (fun (x, y) -> (x + 1, y + 1)) (positions glider));
  check "isolated cell dies" (positions (Life.step (Life.start "" "#")) = []);
  check "empty universe keeps advancing"
    ((Life.advance 3 (Life.start "" ".")).generation = 3);
  check "grid aliases and CRLF"
    (get (L.Ainput.parse ".#\r\n#.\r\n") = get (L.Ainput.parse "01\n10"));
  List.iter
    (fun text ->
      check "bad grid accepted" (Result.is_error (L.Ainput.parse text)))
    [ ""; "01\n1"; "01\n\n10"; "02" ];
  check "nonempty Life code accepted" (Result.is_error (L.Code.parse "30"));
  check "ragged typed grid accepted"
    (Result.is_error (L.make () [ [ true ]; [] ]));
  let saved = L.Snapshot.to_yojson block in
  List.iter
    (fun bad ->
      check "bad Life snapshot accepted"
        (Result.is_error (L.Snapshot.of_yojson bad)))
    [
      `Null;
      replace "generation" (`Int (-1)) saved;
      replace "generation" (`Int 2) saved;
      replace "cells" (`List []) saved;
      replace "cells" (`List [ `List [] ]) saved;
      replace "cells" (`List [ `List [ `Bool true ]; `List [] ]) saved;
    ]

let () =
  elementary ();
  life ();
  print_endline
    "Passed elementary cellular automata and Game of Life transitions, \
     boundaries, codecs and snapshots."
