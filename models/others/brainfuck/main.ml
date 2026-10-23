open Utils_runtime.Machine
module Program = Others.Loop_code
module Byte = Others.Cells.Byte
module Tape = Others.Cells.Byte_tape

module Code = struct
  type t = Program.t

  let parse source =
    let commands = ref [] in
    String.iter
      (fun c ->
        if String.contains "><+-.,[]" c then
          commands := String.make 1 c :: !commands)
      source;
    Program.create ~opening:"[" ~closing:"]" ~allow_empty:true
      (List.rev !commands)

  let print = Program.print
  let to_yojson code = `String (print code)

  let of_yojson = function
    | `String text -> parse text
    | _ -> Error "Expected Brainfuck source"
end

type state = {
  code : Code.t;
  pc : int;
  tape : Tape.t;
  input : Byte.t list;
  output_rev : Byte.t list;
}
[@@deriving yojson]

module Ainput = Others.Cells.Bytes
module Rinput = Utils_data.Text.Unit
module Foutput = Others.Cells.Bytes

module Routput = struct
  type t = int option

  let parse text =
    if String.trim text = "" then Ok None
    else Result.map Option.some (Byte.parse text)

  let print = function None -> "" | Some byte -> Byte.print byte
end

let make code input =
  if not (List.for_all Byte.valid input) then
    Error "Expected input bytes (0..255)"
  else Ok { code; pc = 0; tape = Tape.blank; input; output_rev = [] }

let step s () =
  if s.pc = Program.length s.code then Ok (Halt (List.rev s.output_rev))
  else if s.pc < 0 || s.pc > Program.length s.code then
    Error "Invalid instruction pointer"
  else
    let continue ?(output = None) next = Ok (Continue { next; output }) in
    let next = { s with pc = s.pc + 1 } in
    match Program.token s.code s.pc with
    | ">" -> continue { next with tape = Tape.move_to s.tape Right }
    | "<" ->
        if s.tape.left = [] then Error "Cannot move left of the tape origin"
        else continue { next with tape = Tape.move_to s.tape Left }
    | "+" ->
        continue
          {
            next with
            tape = Tape.head_write s.tape ((s.tape.head + 1) mod 256);
          }
    | "-" ->
        continue
          {
            next with
            tape = Tape.head_write s.tape ((s.tape.head + 255) mod 256);
          }
    | "." ->
        continue ~output:(Some s.tape.head)
          { next with output_rev = s.tape.head :: s.output_rev }
    | "," ->
        let byte, input =
          match s.input with [] -> (0, []) | byte :: rest -> (byte, rest)
        in
        continue { next with tape = Tape.head_write s.tape byte; input }
    | "[" when s.tape.head = 0 ->
        continue { next with pc = Program.matching s.code s.pc + 1 }
    | "]" when s.tape.head <> 0 ->
        continue { next with pc = Program.matching s.code s.pc + 1 }
    | "[" | "]" -> continue next
    | _ -> Error "Invalid Brainfuck instruction"

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    if s.pc < 0 || s.pc > Program.length s.code then
      Error "Invalid instruction pointer"
    else Ok s
end

let render s =
  let cells, head = Tape.into_vec s.tape in
  let open Utils_runtime.Render in
  [
    highlight ~title:"code" ~index:s.pc Fun.id (Program.tokens s.code);
    text ~title:"pc" (string_of_int s.pc);
    highlight ~title:"tape" ~index:head Byte.print cells;
    text ~title:"remaining input (bytes)" (Ainput.print s.input);
    text ~title:"output (bytes)" (Foutput.print (List.rev s.output_rev));
  ]
