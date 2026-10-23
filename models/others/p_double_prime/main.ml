open Utils_runtime.Machine
module Program = Others.Loop_code
module Tape = Others.Cells.Natural_tape

module Code = struct
  type t = Program.t

  let parse source =
    let rec read pos acc =
      if pos = String.length source then
        Program.create ~opening:"(" ~closing:")" ~allow_empty:false
          (List.rev acc)
      else
        match source.[pos] with
        | ' ' | '\t' | '\r' | '\n' -> read (pos + 1) acc
        | ('R' | '(' | ')') as c -> read (pos + 1) (String.make 1 c :: acc)
        | _
          when pos + 2 <= String.length source && String.sub source pos 2 = "λ"
          ->
            read (pos + 2) ("λ" :: acc)
        | _ -> Error (Printf.sprintf "Expected R, λ, ( or ) at byte %d" pos)
    in
    read 0 []

  let print = Program.print
  let to_yojson code = `String (print code)

  let of_yojson = function
    | `String text -> parse text
    | _ -> Error "Expected P double prime source"
end

type input = { modulus : int; tape : Tape.t } [@@deriving yojson]

let validate_input (input : input) =
  if input.modulus < 2 then Error "Alphabet size must be at least 2"
  else if
    not
      (List.for_all
         (fun n -> n >= 0 && n < input.modulus)
         ((input.tape.head :: input.tape.left) @ input.tape.right))
  then Error "Tape symbols must be smaller than the alphabet size"
  else Ok ()

module Ainput = struct
  type t = input

  let parse text =
    let text = String.trim text in
    if text = "" then Ok { modulus = 2; tape = Tape.blank }
    else
      match String.index_opt text '\n' with
      | None -> Error "Expected alphabet size, then a tape on the next line"
      | Some newline ->
          let* modulus =
            Utils_data.Text.Natural_int.parse (String.sub text 0 newline)
          in
          let* tape =
            Tape.parse
              (String.sub text (newline + 1) (String.length text - newline - 1))
          in
          let input = { modulus; tape } in
          let* () = validate_input input in
          Ok input

  let print input = string_of_int input.modulus ^ "\n" ^ Tape.print input.tape
end

type state = { code : Code.t; pc : int; config : input } [@@deriving yojson]

let make code config =
  let* () = validate_input config in
  Ok { code; pc = 0; config }

let step s () =
  if s.pc = Program.length s.code then Ok (Halt s.config.tape)
  else if s.pc < 0 || s.pc > Program.length s.code then
    Error "Invalid instruction pointer"
  else
    let tape = s.config.tape in
    let continue pc tape =
      Ok
        (Continue
           {
             next = { s with pc; config = { s.config with tape } };
             output = ();
           })
    in
    match Program.token s.code s.pc with
    | "R" ->
        (* The last explicitly stored cell is the fixed right boundary. *)
        let tape = if tape.right = [] then tape else Tape.move_to tape Right in
        continue (s.pc + 1) tape
    | "λ" ->
        let value =
          if tape.head = s.config.modulus - 1 then 0 else tape.head + 1
        in
        continue (s.pc + 1) (Tape.move_to (Tape.head_write tape value) Left)
    | "(" when tape.head = 0 -> continue (Program.matching s.code s.pc + 1) tape
    | ")" when tape.head <> 0 ->
        continue (Program.matching s.code s.pc + 1) tape
    | "(" | ")" -> continue (s.pc + 1) tape
    | _ -> Error "Invalid P double prime instruction"

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    let* () = validate_input s.config in
    if s.pc < 0 || s.pc > Program.length s.code then
      Error "Invalid instruction pointer"
    else Ok s
end

let render s =
  let cells, head = Tape.into_vec s.config.tape in
  let open Utils_runtime.Render in
  [
    highlight ~title:"code" ~index:s.pc Fun.id (Program.tokens s.code);
    text ~title:"pc" (string_of_int s.pc);
    text ~title:"alphabet size" (string_of_int s.config.modulus);
    highlight ~title:"tape (infinite to the left; right boundary shown)"
      ~index:head string_of_int cells;
  ]

module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Tape
