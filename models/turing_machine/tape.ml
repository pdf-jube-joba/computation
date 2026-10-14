open Types

let unwrap = Utils_base.Error.get

type t = tape

let blank = { left = []; head = None; right = [] }

let from_vec signs pos =
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
    let head, right = match rest with [] -> (None, []) | h :: t -> (h, t) in
    Ok { left; head; right }

let into_vec t = (List.rev t.left @ (t.head :: t.right), List.length t.left)
let head_write t head = { t with head }

let move_to t = function
  | Constant -> t
  | Left ->
      let head, left =
        match t.left with [] -> (None, []) | h :: xs -> (h, xs)
      in
      { left; head; right = t.head :: t.right }
  | Right ->
      let head, right =
        match t.right with [] -> (None, []) | h :: xs -> (h, xs)
      in
      { left = t.head :: t.left; head; right }

let equal a b =
  let rec side a b =
    match (a, b) with
    | [], xs | xs, [] -> List.for_all (( = ) None) xs
    | x :: xs, y :: ys -> x = y && side xs ys
  in
  a.head = b.head && side a.left b.left && side a.right b.right

let parse_side text =
  if String.trim text = "" then []
  else List.map (fun x -> unwrap (Sign.parse x)) (String.split_on_char ',' text)

let print_side signs = String.concat "," (List.map Sign.print signs)

let parse text =
  Utils_parsing.Parser.protect (fun () ->
      match String.split_on_char '|' text with
      | [ l; h; r ] ->
          {
            left = List.rev (parse_side l);
            head = unwrap (Sign.parse h);
            right = parse_side r;
          }
      | _ -> failwith "Invalid tape format | format ... 0,1,2|3|4,5,6")

let print t =
  print_side (List.rev t.left)
  ^ "|" ^ Sign.print t.head ^ "|" ^ print_side t.right
