module T = One_tape_turing_machine.Main

let get = Utils_base.Error.get
let symbol text = Some (Utils_names.Identifier.make_exn text)
let left_marker = symbol "mt_left"
let right_marker = symbol "mt_right"

(* One column occupies k cells, in tape order. Each cell records its tape,
   symbol and whether the corresponding simulated head is here. *)
let cell tape head sign =
  let payload =
    match sign with
    | None -> "B"
    | Some s -> "S" ^ Utils_names.Identifier.to_string s
  in
  symbol (Printf.sprintf "mt%d_%s_%s" tape (if head then "h" else "p") payload)

let read_cell sign =
  let invalid () = failwith "Invalid encoded tape cell" in
  match sign with
  | None -> invalid ()
  | Some id -> (
      match String.split_on_char '_' (Utils_names.Identifier.to_string id) with
      | tape :: head :: payload ->
          if not (String.starts_with ~prefix:"mt" tape) then invalid ();
          let index =
            int_of_string (String.sub tape 2 (String.length tape - 2))
          in
          if index < 0 then invalid ();
          let head =
            match head with "h" -> true | "p" -> false | _ -> invalid ()
          in
          let payload = String.concat "_" payload in
          let sign =
            if payload = "B" then None
            else if String.starts_with ~prefix:"S" payload then
              Some
                (get
                   (Utils_names.Identifier.make
                      (String.sub payload 1 (String.length payload - 1))))
            else invalid ()
          in
          (index, head, sign)
      | _ -> invalid ())

let encode tapes =
  Utils_base.Error.protect (fun () ->
      if tapes = [] then failwith "Expected at least one tape";
      let left =
        List.fold_left
          (fun n (t : T.tape) -> max n (List.length t.left))
          0 tapes
      in
      let right =
        List.fold_left
          (fun n (t : T.tape) -> max n (List.length t.right))
          0 tapes
      in
      let at (t : T.tape) position =
        let nth xs n = if n < List.length xs then List.nth xs n else None in
        if position = 0 then t.head
        else if position < 0 then nth t.left (-position - 1)
        else nth t.right (position - 1)
      in
      let columns =
        List.init
          (left + right + 1)
          (fun column ->
            let position = column - left in
            List.mapi
              (fun i tape -> cell i (position = 0) (at tape position))
              tapes)
        |> List.concat
      in
      get (T.Tape.from_vec ((left_marker :: columns) @ [ right_marker ]) 0))

(* Padding added by simulation is unobservable on an infinite blank tape.
   Drop only blanks beyond the outermost nonblank cell on either side. *)
let trim_side xs =
  let rec drop = function None :: xs -> drop xs | xs -> xs in
  List.rev (drop (List.rev xs))

let decode (tape : T.tape) =
  Utils_base.Error.protect (fun () ->
      if tape.head <> left_marker || not (List.for_all Option.is_none tape.left)
      then failwith "Expected encoded output head at left boundary";
      let rec contents acc = function
        | [] -> failwith "Missing encoded right boundary"
        | x :: xs when x = right_marker ->
            if not (List.for_all Option.is_none xs) then
              failwith "Data outside encoded right boundary";
            List.rev acc
        | x :: xs -> contents (read_cell x :: acc) xs
      in
      let cells = contents [] tape.right in
      let rec width expected = function
        | (0, _, _) :: _ when expected > 0 -> expected
        | (i, _, _) :: rest when i = expected -> width (expected + 1) rest
        | [] -> expected
        | _ -> failwith "Invalid encoded column order"
      in
      let count = width 0 cells in
      if count = 0 || List.length cells mod count <> 0 then
        failwith "Incomplete encoded column";
      let signs = Array.make count [] in
      let heads = Array.make count None in
      List.iteri
        (fun position (i, head, sign) ->
          if i <> position mod count then
            failwith "Invalid encoded column order";
          if head then (
            if heads.(i) <> None then failwith "Duplicate encoded head";
            heads.(i) <- Some (position / count));
          signs.(i) <- sign :: signs.(i))
        cells;
      Array.to_list
        (Array.mapi
           (fun i signs ->
             let head =
               match heads.(i) with
               | Some h -> h
               | None -> failwith "Missing encoded head"
             in
             let t = get (T.Tape.from_vec (List.rev signs) head) in
             { t with left = trim_side t.left; right = trim_side t.right })
           signs))
