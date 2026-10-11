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
let sign_print = function None -> "-" | Some s -> s

let parse_sign text =
  let text = String.trim text in
  if text = "-" then Ok None
  else Result.map (fun s -> Some s) (Utils.Identifier.make text)

let parse_state text = Utils.Identifier.make (String.trim text)
let direction_print = function Left -> "L" | Right -> "R" | Constant -> "C"

let parse_direction text =
  match String.trim text with
  | "L" -> Ok Left
  | "R" -> Ok Right
  | "C" -> Ok Constant
  | s -> Error ("Invalid direction: " ^ s)

let blank_tape = { left = []; head = None; right = [] }

let tape_from_vec signs pos =
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

let tape_into_vec t = (List.rev t.left @ (t.head :: t.right), List.length t.left)
let head_read t = t.head
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

let tape_equal a b =
  let rec side a b =
    match (a, b) with
    | [], xs | xs, [] -> List.for_all (( = ) None) xs
    | x :: xs, y :: ys -> x = y && side xs ys
  in
  a.head = b.head && side a.left b.left && side a.right b.right

let parse_tape text =
  Utils.Lexer.protect (fun () ->
      let side s =
        if String.trim s = "" then []
        else
          List.map (fun x -> unwrap (parse_sign x)) (String.split_on_char ',' s)
      in
      match String.split_on_char '|' text with
      | [ l; h; r ] ->
          {
            left = List.rev (side l);
            head = unwrap (parse_sign h);
            right = side r;
          }
      | _ -> failwith "Invalid tape format | format ... 0,1,2|3|4,5,6")

let print_tape t =
  String.concat "," (List.map sign_print (List.rev t.left))
  ^ "|" ^ sign_print t.head ^ "|"
  ^ String.concat "," (List.map sign_print t.right)

let definition init_state accepted_state code =
  Utils.Lexer.protect (fun () ->
      ignore (unwrap (parse_state init_state));
      List.iter (fun s -> ignore (unwrap (parse_state s))) accepted_state;
      let seen = Hashtbl.create 16 in
      List.iter
        (fun ((sign, state), (next_sign, next_state, _)) ->
          ignore (unwrap (parse_state state));
          ignore (unwrap (parse_state next_state));
          List.iter
            (fun sign -> ignore (unwrap (parse_sign (sign_print sign))))
            [ sign; next_sign ];
          if List.mem state accepted_state then
            failwith "Code contains accepted state";
          if Hashtbl.mem seen (sign, state) then
            failwith "Duplicate transition for key state/sign";
          Hashtbl.add seen (sign, state) ())
        code;
      { init_state; accepted_state; code })

let parse_code text =
  Utils.Lexer.protect (fun () ->
      match String.split_on_char '\n' text with
      | init :: accepted :: lines ->
          let init = unwrap (parse_state init) in
          let accepted =
            List.map
              (fun s -> unwrap (parse_state s))
              (String.split_on_char ',' accepted)
          in
          let entries =
            List.filter_map
              (fun line ->
                let line =
                  String.trim (List.hd (String.split_on_char '#' line))
                in
                if line = "" then None
                else
                  match String.split_on_char ',' line with
                  | a :: b :: c :: d :: e :: _ ->
                      Some
                        ( (unwrap (parse_sign a), unwrap (parse_state b)),
                          ( unwrap (parse_sign c),
                            unwrap (parse_state d),
                            unwrap (parse_direction e) ) )
                  | _ -> failwith ("Invalid code entry: " ^ line))
              lines
          in
          unwrap (definition init accepted entries)
      | [] | [ _ ] -> failwith "Missing initial or accepted state line")

let definition_signs d =
  List.concat_map (fun ((a, _), (b, _, _)) -> [ a; b ]) d.code

let definition_states d =
  (d.init_state :: d.accepted_state)
  @ List.concat_map (fun ((_, a), (_, b, _)) -> [ a; b ]) d.code

let get_now_entry d key =
  let rec loop i = function
    | [] -> None
    | (k, value) :: xs -> if k = key then Some (i, value) else loop (i + 1) xs
  in
  loop 0 d.code

let get_next_state d key = Option.map snd (get_now_entry d key)

let print_code d =
  d.init_state ^ "\n"
  ^ String.concat "," d.accepted_state
  ^ "\n"
  ^ String.concat ""
      (List.map
         (fun ((a, b), (c, d, e)) ->
           String.concat ","
             [ sign_print a; b; sign_print c; d; direction_print e ]
           ^ "\n")
         d.code)

let parse_ainput = parse_tape
let parse_rinput = empty_input

let next_code s =
  if List.mem s.current_state s.definition.accepted_state then None
  else get_now_entry s.definition (s.tape.head, s.current_state)

let now_state s = s.current_state
let now_tape s = s.tape
let is_accepted s = List.mem s.current_state s.definition.accepted_state
let is_terminate s = next_code s = None

let make definition tape =
  Ok { definition; current_state = definition.init_state; tape }

let one_step s =
  match next_code s with
  | None -> s
  | Some (_, (head, current_state, dir)) ->
      { s with current_state; tape = move_to (head_write s.tape head) dir }

let step s () =
  let next = one_step s in
  if is_terminate next then Ok (Halt next.tape)
  else Ok (Continue { next; output = () })

let steps s n =
  let rec loop s i =
    if i = n then Ok s
    else if is_terminate s then Error i
    else loop (one_step s) (i + 1)
  in
  loop s 0

let result s = if is_terminate s then Ok s.tape else Error "not terminated"
let snapshot = state_to_yojson

let restore json =
  let* s = state_of_yojson json in
  let* _ =
    definition s.definition.init_state s.definition.accepted_state
      s.definition.code
  in
  let* _ = parse_state s.current_state in
  let* _ = parse_tape (print_tape s.tape) in
  Ok s

let render_tape t =
  let signs, pos = tape_into_vec t in
  Utils.Render.container ~title:"tape" ~orientation:"horizontal"
    (List.mapi
       (fun i s ->
         if i = pos then
           Utils.Render.text ~class_name:"highlight" (sign_print s)
         else Utils.Render.text (sign_print s))
       signs)

let render s =
  let index = Option.map fst (next_code s) in
  Utils.Render.state
    [
      Utils.Render.text_table ~title:"code" ?highlight:index
        ~columns:
          [ "key_sign"; "key_state"; "next_sign"; "next_state"; "direction" ]
        (List.map
           (fun ((a, b), (c, d, e)) ->
             [ sign_print a; b; sign_print c; d; direction_print e ])
           s.definition.code);
      Utils.Render.text ~title:"state" s.current_state;
      render_tape s.tape;
    ]

let routput_json () = `String ""
let foutput_json t = `String (print_tape t)

module Machine = struct
  type nonrec code = code
  type nonrec ainput = ainput
  type nonrec rinput = rinput
  type nonrec routput = routput
  type nonrec foutput = foutput
  type nonrec state = state

  let parse_code = parse_code
  let parse_ainput = parse_ainput
  let parse_rinput = parse_rinput
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
  let routput_json = routput_json
  let foutput_json = foutput_json
end

module Tape = struct
  type t = tape

  let blank = blank_tape
  let from_vec = tape_from_vec
  let into_vec = tape_into_vec
  let head_read = head_read
  let head_write = head_write
  let move_to = move_to
  let equal = tape_equal
  let parse = parse_tape
  let print = print_tape
end

module Sign = struct
  type t = sign

  let blank = None
  let parse = parse_sign
  let print = sign_print
end

module Direction = struct
  type t = direction = Right | Constant | Left

  let parse = parse_direction
  let print = direction_print
end

module State = struct
  type t = Utils.Identifier.t

  let parse = parse_state
  let print s = s
end

module Definition = struct
  type t = code

  let make = definition
  let parse = parse_code
  let print = print_code
  let init_state d = d.init_state
  let accepted_state d = d.accepted_state
  let code d = d.code
  let signs = definition_signs
  let states = definition_states
  let get_now_entry = get_now_entry
  let get_next_state = get_next_state
end
