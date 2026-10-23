module Source = Multi_tape_turing_machine.Main
module Target = One_tape_turing_machine.Main
module E = Encoding
open Target

let compile (d : Source.Code.t) =
  Utils_base.Error.protect (fun () ->
      let get = Utils_base.Error.get in
      get (Source.Code.validate d);
      let supply = ref State_id.Supply.empty in
      let names = ref [] in
      let serial = ref 0 in
      let fresh hint =
        let id = State_id.Supply.take supply in
        let name =
          Utils_names.Identifier.make_exn (Printf.sprintf "%s_%d" hint !serial)
        in
        incr serial;
        names := (id, name) :: !names;
        id
      in
      let entries = ref [] in
      let emit source read write target dir =
        entries := ((read, source), (write, target, dir)) :: !entries
      in
      let keep source sign target dir = emit source sign sign target dir in
      let cells =
        List.init d.tape_count (fun i ->
            List.concat_map
              (fun sign -> [ (i, false, sign); (i, true, sign) ])
              d.alphabet)
        |> List.concat
      in
      let each_cell f =
        List.iter (fun (i, head, sign) -> f (E.cell i head sign)) cells
      in
      let states =
        List.map
          (fun (id, name) ->
            (id, fresh (Utils_names.Identifier.to_string name)))
          d.state_names
      in
      let state id = List.assoc id states in
      let stuck = fresh "stuck" in
      (* Every phase starts at the left boundary. Rewinds can be shared by
         phases with the same continuation. *)
      let rewinds = Hashtbl.create 32 in
      let rewind target =
        match Hashtbl.find_opt rewinds target with
        | Some s -> s
        | None ->
            let s = fresh "rewind" in
            Hashtbl.add rewinds target s;
            each_cell (fun sign -> keep s sign s Left);
            keep s E.right_marker s Left;
            keep s E.left_marker target Constant;
            s
      in
      let begin_scan start scan = keep start E.left_marker scan Right in
      (* Add one blank column at each end before writing. A head can move at
         most one column, so subsequent k-cell moves never cross a boundary. *)
      let extend start target =
        let sweep = fresh "extend_right_scan" in
        let left_states =
          Array.init d.tape_count (fun _ -> fresh "extend_left")
        in
        emit start E.left_marker
          (E.cell (d.tape_count - 1) false None)
          left_states.(d.tape_count - 1)
          Left;
        for i = d.tape_count - 1 downto 1 do
          emit left_states.(i) None
            (E.cell (i - 1) false None)
            left_states.(i - 1)
            Left
        done;
        emit left_states.(0) None E.left_marker sweep Right;
        each_cell (fun sign -> keep sweep sign sweep Right);
        let right_states =
          Array.init d.tape_count (fun _ -> fresh "extend_right")
        in
        emit sweep E.right_marker (E.cell 0 false None) right_states.(0) Right;
        for i = 0 to d.tape_count - 2 do
          emit right_states.(i) None
            (E.cell (i + 1) false None)
            right_states.(i + 1)
            Right
        done;
        emit
          right_states.(d.tape_count - 1)
          None E.right_marker (rewind target) Left
      in
      List.iter
        (fun (source, _) ->
          let rules = List.filter (fun ((_, q), _) -> q = source) d.code in
          let starts =
            List.mapi
              (fun i _ -> if i = 0 then state source else fresh "candidate")
              rules
          in
          List.iteri
            (fun rule_index ((reads, _), (writes, next, dirs)) ->
              let fail =
                if rule_index + 1 = List.length starts then stuck
                else List.nth starts (rule_index + 1)
              in
              let expand = fresh "expand" in
              let checks =
                Array.init d.tape_count (fun i ->
                    if i = 0 then List.nth starts rule_index else fresh "check")
              in
              (* Match the entire read vector before changing any tape. A
                 mismatched candidate rewinds without changing the encoding. *)
              List.iteri
                (fun i expected ->
                  let scan = fresh "read_head" in
                  let success =
                    if i + 1 = d.tape_count then expand else checks.(i + 1)
                  in
                  begin_scan checks.(i) scan;
                  List.iter
                    (fun (j, head, sign) ->
                      let encoded = E.cell j head sign in
                      if i = j && head then
                        keep scan encoded
                          (rewind (if sign = expected then success else fail))
                          Left
                      else keep scan encoded scan Right)
                    cells;
                  keep scan E.right_marker (rewind fail) Left)
                reads;
              let updates = Array.init d.tape_count (fun _ -> fresh "update") in
              extend expand updates.(0);
              List.iteri
                (fun i (write, dir) ->
                  let scan = fresh "write_head" in
                  let next =
                    if i + 1 = d.tape_count then state next else updates.(i + 1)
                  in
                  let back = rewind next in
                  begin_scan updates.(i) scan;
                  let destination =
                    if dir = Constant then back
                    else
                      let mark = fresh "mark_head" in
                      List.iter
                        (fun sign ->
                          emit mark (E.cell i false sign) (E.cell i true sign)
                            back Left)
                        d.alphabet;
                      (* Writing the old cell performs the first move. *)
                      let remaining = ref mark in
                      for _ = 1 to d.tape_count - 1 do
                        let move = fresh "move_head" in
                        each_cell (fun sign -> keep move sign !remaining dir);
                        remaining := move
                      done;
                      !remaining
                  in
                  List.iter
                    (fun (j, head, sign) ->
                      let encoded = E.cell j head sign in
                      if i = j && head then
                        emit scan encoded
                          (E.cell i (dir = Constant) write)
                          destination
                          (if dir = Constant then Left else dir)
                      else keep scan encoded scan Right)
                    cells)
                (List.combine writes dirs))
            rules)
        d.state_names;
      get
        (Code.of_ids ~state_names:(List.rev !names) (state d.init_state)
           (List.map state d.accepted_state)
           (List.rev !entries)))

let encode_ainput = E.encode
let encode_rinput () = Ok ()
let decode_routput () = Ok ()
let decode_foutput = E.decode
