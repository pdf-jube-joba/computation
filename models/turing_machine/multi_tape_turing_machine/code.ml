open Types

type t = code

let get = Utils_base.Error.get
let state_name d id = List.assoc id d.state_names

let validate d =
  Utils_parsing.Parser.protect (fun () ->
      if d.tape_count < 1 then failwith "Tape count must be positive";
      if not (List.mem Sign.default d.alphabet) then
        failwith "Alphabet must contain the blank symbol (-)";
      if
        List.length d.alphabet
        <> List.length (List.sort_uniq compare d.alphabet)
      then failwith "Duplicate alphabet symbol";
      List.iter (fun s -> ignore (get (Sign.parse (Sign.print s)))) d.alphabet;
      let ids = List.map fst d.state_names in
      let known_ids = State_id.Set.of_list ids in
      if List.length ids <> State_id.Set.cardinal known_ids then
        failwith "Duplicate state ID";
      Utils_names.Identifier.check_unique
        ~message:(fun name ->
          "Duplicate state name: " ^ Utils_names.Identifier.to_string name)
        (List.map snd d.state_names);
      let known id =
        if not (State_id.Set.mem id known_ids) then failwith "Unknown state ID"
      in
      known d.init_state;
      List.iter known d.accepted_state;
      let seen = Hashtbl.create 16 in
      List.iter
        (fun ((reads, source), (writes, target, directions)) ->
          known source;
          known target;
          if List.mem source d.accepted_state then
            failwith "Code contains accepted state";
          if
            List.length reads <> d.tape_count
            || List.length writes <> d.tape_count
            || List.length directions <> d.tape_count
          then failwith "Transition vector length differs from tape count";
          List.iter
            (fun sign ->
              if not (List.mem sign d.alphabet) then
                failwith ("Symbol outside alphabet: " ^ Sign.print sign))
            (reads @ writes);
          if Hashtbl.mem seen (reads, source) then
            failwith "Duplicate transition for state/read vector";
          Hashtbl.add seen (reads, source) ())
        d.code)

let of_ids ~tape_count ~alphabet ~state_names init_state accepted_state code =
  let d =
    { tape_count; alphabet; init_state; accepted_state; code; state_names }
  in
  Result.map (fun () -> d) (validate d)

let make ~tape_count ~alphabet init accepted entries =
  Utils_parsing.Parser.protect (fun () ->
      let supply = ref State_id.Supply.empty in
      let ids = Hashtbl.create 16 in
      let names = ref [] in
      let state name =
        match Hashtbl.find_opt ids name with
        | Some id -> id
        | None ->
            let id = State_id.Supply.take supply in
            Hashtbl.add ids name id;
            names := (id, name) :: !names;
            id
      in
      let init = state init in
      let accepted = List.map state accepted in
      let entries =
        List.map
          (fun ((reads, source), (writes, target, dirs)) ->
            let source = state source in
            let target = state target in
            ((reads, source), (writes, target, dirs)))
          entries
      in
      get
        (of_ids ~tape_count ~alphabet ~state_names:(List.rev !names) init
           accepted entries))

let words text =
  String.split_on_char ' ' (String.map (function '\t' -> ' ' | c -> c) text)
  |> List.filter (fun s -> s <> "")

let parse text =
  Utils_parsing.Parser.protect (fun () ->
      let lines =
        String.split_on_char '\n' text
        |> List.map (fun line ->
            String.trim (Utils_base.String_ext.strip_line_comment "#" line))
      in
      let header prefix line =
        if not (String.starts_with ~prefix line) then
          failwith ("Expected " ^ prefix);
        String.trim
          (String.sub line (String.length prefix)
             (String.length line - String.length prefix))
      in
      let state s = get (Utils_names.Identifier.make (String.trim s)) in
      match lines with
      | count :: alphabet :: init :: accepted :: entries ->
          let tape_count = int_of_string (header "tapes:" count) in
          let alphabet =
            String.split_on_char ',' (header "alphabet:" alphabet)
            |> List.map (fun s -> get (Sign.parse s))
          in
          let accepted =
            if accepted = "" then []
            else List.map state (String.split_on_char ',' accepted)
          in
          let vector parse s = List.map (fun s -> get (parse s)) (words s) in
          let entries =
            List.filter_map
              (fun line ->
                if line = "" then None
                else
                  match String.split_on_char ',' line with
                  | [ reads; source; writes; target; dirs ] ->
                      Some
                        ( (vector Sign.parse reads, state source),
                          ( vector Sign.parse writes,
                            state target,
                            vector Direction.parse dirs ) )
                  | _ -> failwith ("Invalid transition: " ^ line))
              entries
          in
          get (make ~tape_count ~alphabet (state init) accepted entries)
      | _ ->
          failwith
            "Missing tape count, alphabet, initial or accepted state line")

let print d =
  let name id = Utils_names.Identifier.to_string (state_name d id) in
  let vector print xs = String.concat " " (List.map print xs) in
  Printf.sprintf "tapes: %d\nalphabet: %s\n%s\n%s\n" d.tape_count
    (String.concat "," (List.map Sign.print d.alphabet))
    (name d.init_state)
    (String.concat "," (List.map name d.accepted_state))
  ^ String.concat ""
      (List.map
         (fun ((reads, source), (writes, target, dirs)) ->
           String.concat ","
             [
               vector Sign.print reads;
               name source;
               vector Sign.print writes;
               name target;
               vector Direction.print dirs;
             ]
           ^ "\n")
         d.code)

let find_transition d key =
  let rec loop i = function
    | [] -> None
    | (k, value) :: rest ->
        if k = key then Some (i, value) else loop (i + 1) rest
  in
  loop 0 d.code
