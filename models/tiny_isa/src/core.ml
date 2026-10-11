open Isa_common

type code = N.t list [@@deriving yojson]

type register_memory_image = { register : N.t array; memory : N.t list }
[@@deriving yojson]

type ainput = register_memory_image
type rinput = unit
type routput = unit
type foutput = register_memory_image

type state = {
  regs : N.t array;
  flag : bool;
  code_len : int;
  memory : N.t array;
  halted : bool;
}
[@@deriving yojson]

let parse_code = parse_numbers
let print = print_numbers

let parse_ainput text =
  Result.map (fun (register, memory) -> { register; memory }) (parse_image text)

let parse_rinput = Utils.Machine.empty_input

let make code (input : ainput) =
  Utils.Lexer.protect (fun () ->
      validate_regs input.register;
      {
        regs = Array.copy input.register;
        flag = false;
        code_len = List.length code;
        memory = Array.of_list (code @ input.memory);
        halted = false;
      })

let output s =
  {
    register = Array.copy s.regs;
    memory =
      Array.to_list
        (Array.sub s.memory s.code_len (Array.length s.memory - s.code_len));
  }

let decode = decode
let encode = encode

let step s () =
  Utils.Lexer.protect (fun () ->
      if s.halted then Utils.Machine.Halt (output s)
      else
        let pc = value (N.to_int s.regs.(0)) in
        let inst = value (decode (read s.memory pc)) in
        let regs, flag, memory, halted =
          execute ~resolve:Fun.id inst s.regs s.flag s.memory
        in
        let next = { s with regs; flag; memory; halted } in
        if halted then Utils.Machine.Halt (output next)
        else Utils.Machine.Continue { next; output = () })

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      Utils.Lexer.protect (fun () ->
          validate_regs s.regs;
          if s.code_len < 0 || s.code_len > Array.length s.memory then
            failwith "Invalid code length";
          s))

let render s =
  let pc = match N.to_int s.regs.(0) with Ok n -> Some n | _ -> None in
  let next =
    match pc with
    | Some i when i < Array.length s.memory -> (
        match decode s.memory.(i) with
        | Ok inst -> inst_text N.to_string inst
        | _ -> "halt")
    | _ -> "halt"
  in
  let memory =
    table "memory" [ "addr"; "value" ]
      (Array.to_list
         (Array.mapi (fun i n -> [ string_of_int i; N.to_string n ]) s.memory))
  in
  let memory =
    match memory with
    | `Assoc fields ->
        let rows = List.assoc "rows" fields in
        let rows =
          match rows with
          | `List rows ->
              `List
                (List.mapi
                   (fun i row ->
                     if pc = Some i then
                       match row with
                       | `Assoc fields ->
                           `Assoc
                             (fields @ [ ("className", `String "highlight") ])
                       | _ -> row
                     else row)
                   rows)
          | rows -> rows
        in
        `Assoc (("rows", rows) :: List.remove_assoc "rows" fields)
    | _ -> memory
  in
  Utils.Render.state
    [
      table "status" [ "field"; "value" ]
        [
          [ "pc"; (match pc with Some n -> string_of_int n | None -> "?") ];
          [ "flag"; string_of_bool s.flag ];
          [ "next"; next ];
          [ "halted"; string_of_bool s.halted ];
        ];
      registers s.regs;
      memory;
    ]

let routput_json () = `String ""
let foutput_json (o : foutput) = `String (print_image o.register o.memory)

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
