module type VALUE = sig
  type t [@@deriving yojson]

  val default : t
  val equal : t -> t -> bool
  val parse : string -> (t, string) result
  val print : t -> string
end

module Direction = struct
  type t = Utils_data.Tape.direction = Right | Constant | Left

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

  type direction = Utils_data.Tape.direction = Right | Constant | Left
  [@@deriving yojson]

  type sign = Value.t [@@deriving yojson]
  type t = sign Utils_data.Tape.t [@@deriving yojson]

  let unwrap = Utils_base.Error.get
  let blank = Utils_data.Tape.blank ~default:Value.default
  let from_vec = Utils_data.Tape.from_vec ~default:Value.default
  let into_vec = Utils_data.Tape.into_vec
  let head_write = Utils_data.Tape.head_write
  let move_to = Utils_data.Tape.move_to ~default:Value.default
  let equal = Utils_data.Tape.equal ~default:Value.default ~equal:Value.equal

  let parse_side text =
    if String.trim text = "" then []
    else
      List.map (fun x -> unwrap (Value.parse x)) (String.split_on_char ',' text)

  let print_side values = String.concat "," (List.map Value.print values)

  let parse text =
    Utils_parsing.Parser.protect (fun () ->
        match String.split_on_char '|' text with
        | [ l; h; r ] ->
            {
              Utils_data.Tape.left = List.rev (parse_side l);
              head = unwrap (Value.parse h);
              right = parse_side r;
            }
        | _ -> failwith "Invalid tape format | format ... 0,1,2|3|4,5,6")

  let print (t : t) =
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
