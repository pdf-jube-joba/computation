type direction = Right | Constant | Left [@@deriving yojson]

(* Both sides are stored nearest to the head first. *)
type 'a t = { left : 'a list; head : 'a; right : 'a list } [@@deriving yojson]

let blank ~default = { left = []; head = default; right = [] }

let from_vec ~default signs pos =
  if pos < 0 || pos > List.length signs then Error "Position out of bounds"
  else
    let rec split n acc xs =
      if n = 0 then (acc, xs)
      else
        match xs with
        | h :: t -> split (n - 1) (h :: acc) t
        | [] -> assert false
    in
    let left, rest = split pos [] signs in
    let head, right =
      match rest with [] -> (default, []) | h :: t -> (h, t)
    in
    Ok { left; head; right }

let into_vec t = (List.rev t.left @ (t.head :: t.right), List.length t.left)
let head_write t head = { t with head }

let move_to ~default t = function
  | Constant -> t
  | Left ->
      let head, left =
        match t.left with [] -> (default, []) | h :: xs -> (h, xs)
      in
      { left; head; right = t.head :: t.right }
  | Right ->
      let head, right =
        match t.right with [] -> (default, []) | h :: xs -> (h, xs)
      in
      { left = t.head :: t.left; head; right }

let equal ~default ~equal a b =
  let rec side a b =
    match (a, b) with
    | [], xs | xs, [] -> List.for_all (equal default) xs
    | x :: xs, y :: ys -> equal x y && side xs ys
  in
  equal a.head b.head && side a.left b.left && side a.right b.right
