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

module type VALUE = sig
  type t [@@deriving yojson]

  val default : t
  val equal : t -> t -> bool
  val parse : string -> (t, string) result
  val print : t -> string
end

module Direction = struct
  type t = direction = Right | Constant | Left

  let parse text =
    match String.trim text with
    | "L" -> Ok Left
    | "R" -> Ok Right
    | "C" -> Ok Constant
    | s -> Error ("Invalid direction: " ^ s)

  let print = function Left -> "L" | Right -> "R" | Constant -> "C"
end

module Make (Value : VALUE) = struct
  module Sign = Value
  module Direction = Direction

  type nonrec direction = direction = Right | Constant | Left

  let direction_to_yojson = direction_to_yojson
  let direction_of_yojson = direction_of_yojson

  type sign = Value.t [@@deriving yojson]
  type nonrec t = sign t

  let to_yojson tape = to_yojson sign_to_yojson tape
  let of_yojson json = of_yojson sign_of_yojson json
  let blank = blank ~default:Value.default
  let from_vec = from_vec ~default:Value.default
  let into_vec = into_vec
  let head_write = head_write
  let move_to = move_to ~default:Value.default
  let equal = equal ~default:Value.default ~equal:Value.equal

  let parse text =
    let parse_side text =
      if String.trim text = "" then []
      else
        List.map
          (fun x -> Utils_base.Error.get (Value.parse x))
          (String.split_on_char ',' text)
    in
    Utils_parsing.Parser.protect (fun () ->
        match String.split_on_char '|' text with
        | [ l; h; r ] ->
            {
              left = List.rev (parse_side l);
              head = Utils_base.Error.get (Value.parse h);
              right = parse_side r;
            }
        | _ -> failwith "Invalid tape format | format ... 0,1,2|3|4,5,6")

  let print (t : t) =
    let print_side values = String.concat "," (List.map Value.print values) in
    print_side (List.rev t.left)
    ^ "|" ^ Value.print t.head ^ "|" ^ print_side t.right
end

module Identifier = Make (struct
  type t = Utils_names.Identifier.t option [@@deriving yojson]

  let default = None
  let equal = Option.equal Utils_names.Identifier.equal

  let parse text =
    let text = String.trim text in
    if text = "-" then Ok None
    else Result.map (fun s -> Some s) (Utils_names.Identifier.make text)

  let print = function
    | None -> "-"
    | Some s -> Utils_names.Identifier.to_string s
end)
