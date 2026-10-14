open Types

let unwrap = Utils_base.Error.get

type t = code

let state_name d =
  let names = State_id.Map.of_seq (List.to_seq d.state_names) in
  fun id ->
    match State_id.Map.find_opt id names with
    | Some name -> name
    | None -> invalid_arg "Unknown state ID"

let validate d =
  Utils_parsing.Parser.protect (fun () ->
      let ids = List.map fst d.state_names in
      let known_ids = State_id.Set.of_list ids in
      if List.length ids <> State_id.Set.cardinal known_ids then
        failwith "Duplicate state ID";
      Utils_names.Identifier.check_unique
        ~message:(fun name -> "Duplicate state name: " ^ name)
        (List.map snd d.state_names);
      List.iter
        (fun (_, name) -> ignore (unwrap (State.parse name)))
        d.state_names;
      let known id =
        if not (State_id.Set.mem id known_ids) then failwith "Unknown state ID"
      in
      known d.init_state;
      List.iter known d.accepted_state;
      let seen = Hashtbl.create 16 in
      List.iter
        (fun ((sign, state), (next_sign, next_state, _)) ->
          known state;
          known next_state;
          List.iter
            (fun sign -> ignore (unwrap (Sign.parse (Sign.print sign))))
            [ sign; next_sign ];
          if List.mem state d.accepted_state then
            failwith "Code contains accepted state";
          if Hashtbl.mem seen (sign, state) then
            failwith "Duplicate transition for key state/sign";
          Hashtbl.add seen (sign, state) ())
        d.code)

let of_ids ~state_names init_state accepted_state code =
  let definition = { init_state; accepted_state; code; state_names } in
  Result.map (fun () -> definition) (validate definition)

(* Named code is an input format. Execution and composition use state IDs. *)
let make init_state accepted_state code =
  Utils_parsing.Parser.protect (fun () ->
      let supply = ref State_id.Supply.empty in
      let ids = ref Utils_names.Identifier.Map.empty in
      let names = ref [] in
      let state name =
        let name = unwrap (State.parse name) in
        match Utils_names.Identifier.Map.find_opt name !ids with
        | Some id -> id
        | None ->
            let id = State_id.Supply.take supply in
            ids := Utils_names.Identifier.Map.add name id !ids;
            names := (id, name) :: !names;
            id
      in
      let init_state = state init_state in
      let accepted_state = List.map state accepted_state in
      let code =
        List.map
          (fun ((sign, source), (next_sign, target, direction)) ->
            let source = state source in
            let target = state target in
            ((sign, source), (next_sign, target, direction)))
          code
      in
      let d =
        { init_state; accepted_state; code; state_names = List.rev !names }
      in
      unwrap (validate d);
      d)

let parse text =
  Utils_parsing.Parser.protect (fun () ->
      match String.split_on_char '\n' text with
      | init :: accepted :: lines ->
          let init = unwrap (State.parse init) in
          let accepted =
            List.map
              (fun s -> unwrap (State.parse s))
              (String.split_on_char ',' accepted)
          in
          let entries =
            List.filter_map
              (fun line ->
                let line =
                  String.trim
                    (Utils_base.String_ext.strip_line_comment "#" line)
                in
                if line = "" then None
                else
                  match String.split_on_char ',' line with
                  | a :: b :: c :: d :: e :: _ ->
                      Some
                        ( (unwrap (Sign.parse a), unwrap (State.parse b)),
                          ( unwrap (Sign.parse c),
                            unwrap (State.parse d),
                            unwrap (Direction.parse e) ) )
                  | _ -> failwith ("Invalid code entry: " ^ line))
              lines
          in
          unwrap (make init accepted entries)
      | [] | [ _ ] -> failwith "Missing initial or accepted state line")

let print d =
  let name = state_name d in
  name d.init_state ^ "\n"
  ^ String.concat "," (List.map name d.accepted_state)
  ^ "\n"
  ^ String.concat ""
      (List.map
         (fun ((a, b), (c, target, e)) ->
           String.concat ","
             [
               Sign.print a;
               name b;
               Sign.print c;
               name target;
               Direction.print e;
             ]
           ^ "\n")
         d.code)

let signs d = List.concat_map (fun ((a, _), (b, _, _)) -> [ a; b ]) d.code

let states d =
  (d.init_state :: d.accepted_state)
  @ List.concat_map (fun ((_, a), (_, b, _)) -> [ a; b ]) d.code

let find_transition d key =
  let rec loop i = function
    | [] -> None
    | (k, value) :: xs -> if k = key then Some (i, value) else loop (i + 1) xs
  in
  loop 0 d.code
