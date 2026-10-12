open Utils.Machine

type direction = Right | Constant | Left [@@deriving yojson]
type sign = Utils.Identifier.t option [@@deriving yojson]

type tape = { left : sign list; head : sign; right : sign list }
[@@deriving yojson]

(* Both sides are stored nearest to the head first. *)
type code_entry =
  (sign * Utils.Identifier.t) * (sign * Utils.Identifier.t * direction)
[@@deriving yojson]

type code = {
  init_state : Utils.Identifier.t;
  accepted_state : Utils.Identifier.t list;
  code : code_entry list;
}
[@@deriving yojson]

type ainput = tape
type rinput = unit
type routput = unit
type foutput = tape

type state = {
  definition : code;
  current_state : Utils.Identifier.t;
  tape : tape;
}
[@@deriving yojson]

let unwrap = function Ok v -> v | Error e -> failwith e

module Sign = struct
  type t = sign

  let blank = None

  let parse text =
    let text = String.trim text in
    if text = "-" then Ok None
    else Result.map (fun s -> Some s) (Utils.Identifier.make text)

  let print = function None -> "-" | Some s -> s
end

module State = struct
  type t = Utils.Identifier.t

  let parse text = Utils.Identifier.make (String.trim text)
  let print text = text
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

module Tape = struct
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
    else
      List.map (fun x -> unwrap (Sign.parse x)) (String.split_on_char ',' text)

  let print_side signs = String.concat "," (List.map Sign.print signs)

  let parse text =
    Utils.Lexer.protect (fun () ->
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

  let render t =
    let signs, pos = into_vec t in
    Utils.Render.container ~title:"tape" ~orientation:"horizontal"
      (List.mapi
         (fun i s ->
           if i = pos then
             Utils.Render.text ~class_name:"highlight" (Sign.print s)
           else Utils.Render.text (Sign.print s))
         signs)
end

module Code = struct
  type t = code

  let make init_state accepted_state code =
    Utils.Lexer.protect (fun () ->
        ignore (unwrap (State.parse init_state));
        List.iter (fun s -> ignore (unwrap (State.parse s))) accepted_state;
        let seen = Hashtbl.create 16 in
        List.iter
          (fun ((sign, state), (next_sign, next_state, _)) ->
            ignore (unwrap (State.parse state));
            ignore (unwrap (State.parse next_state));
            List.iter
              (fun sign -> ignore (unwrap (Sign.parse (Sign.print sign))))
              [ sign; next_sign ];
            if List.mem state accepted_state then
              failwith "Code contains accepted state";
            if Hashtbl.mem seen (sign, state) then
              failwith "Duplicate transition for key state/sign";
            Hashtbl.add seen (sign, state) ())
          code;
        { init_state; accepted_state; code })

  let parse text =
    Utils.Lexer.protect (fun () ->
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
                    String.trim (Utils.String_ext.strip_line_comment "#" line)
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
    d.init_state ^ "\n"
    ^ String.concat "," d.accepted_state
    ^ "\n"
    ^ String.concat ""
        (List.map
           (fun ((a, b), (c, d, e)) ->
             String.concat ","
               [ Sign.print a; b; Sign.print c; d; Direction.print e ]
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
end

let next_code s =
  if List.mem s.current_state s.definition.accepted_state then None
  else Code.find_transition s.definition (s.tape.head, s.current_state)

let is_accepted s = List.mem s.current_state s.definition.accepted_state
let is_terminate s = next_code s = None

let make definition tape =
  Ok { definition; current_state = definition.init_state; tape }

let one_step s =
  match next_code s with
  | None -> s
  | Some (_, (head, current_state, dir)) ->
      {
        s with
        current_state;
        tape = Tape.move_to (Tape.head_write s.tape head) dir;
      }

let step s () =
  let next = one_step s in
  if is_terminate next then Ok (Halt next.tape)
  else Ok (Continue { next; output = () })

let result s = if is_terminate s then Ok s.tape else Error "not terminated"
let snapshot = state_to_yojson

let restore json =
  let* s = state_of_yojson json in
  let* _ =
    Code.make s.definition.init_state s.definition.accepted_state
      s.definition.code
  in
  let* _ = State.parse s.current_state in
  let* _ = Tape.parse (Tape.print s.tape) in
  Ok s

let render s =
  let index = Option.map fst (next_code s) in
  Utils.Render.state
    [
      Utils.Render.text_table ~title:"code" ?highlight:index
        ~columns:
          [ "key_sign"; "key_state"; "next_sign"; "next_state"; "direction" ]
        (List.map
           (fun ((a, b), (c, d, e)) ->
             [ Sign.print a; b; Sign.print c; d; Direction.print e ])
           s.definition.code);
      Utils.Render.text ~title:"state" s.current_state;
      Tape.render s.tape;
    ]

module Ainput = Tape
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput
