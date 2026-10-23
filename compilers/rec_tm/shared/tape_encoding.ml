module N = Utils_base.Number
module T = Utils_data.Tape.Identifier

let b = None
let l = Some (Utils_names.Identifier.make_exn "l")
let x = Some (Utils_names.Identifier.make_exn "x")
let alphabet = [ b; l; x ]

let write tuple =
  Utils_base.Error.protect (fun () ->
      let signs =
        x
        :: List.concat_map
             (fun n ->
               let n = Utils_base.Error.get (N.to_int n) in
               b :: List.init n (fun _ -> l))
             tuple
        @ [ x ]
      in
      Utils_base.Error.get (T.from_vec signs 0))

let write_usize tuple = write (List.map N.of_int tuple)

let read_right_one tape =
  if tape.Utils_data.Tape.head <> x then None
  else
    let rec take acc = function
      | [] -> List.rev acc
      | s :: _ when s = x -> List.rev acc
      | s :: xs -> take (s :: acc) xs
    in
    let signs = take [] tape.right in
    let rec split current acc = function
      | [] -> List.rev (current :: acc)
      | s :: xs when s = b -> split 0 (current :: acc) xs
      | _ :: xs -> split (current + 1) acc xs
    in
    match split 0 [] signs with
    | [] -> Some []
    | _ :: ns -> Some (List.map N.of_int ns)

let read_right_one_usize t =
  Option.map
    (List.map (fun n -> Utils_base.Error.get (N.to_int n)))
    (read_right_one t)
