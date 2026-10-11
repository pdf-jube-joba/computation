module N = Utils.Number

let value = function Ok v -> v | Error e -> failwith e

type 'a inst =
  | Nop
  | Halt
  | Rst
  | Ld of int * int
  | St of int * int
  | Mov of bool * int * int
  | Ldi of bool * int * 'a
  | Add of bool * int * int
  | Sub of bool * int * int
  | Addi of bool * int * 'a
  | Subi of bool * int * 'a
  | Eq of int * int
  | Lt of int * int
  | Gt of int * int
[@@deriving yojson]

let read array addr =
  if addr < 0 then failwith "Negative address"
  else if addr < Array.length array then array.(addr)
  else N.zero

let write array addr v =
  if addr < 0 || addr >= Sys.max_array_length then
    failwith "Memory address out of range";
  let out = Array.make (max (Array.length array) (addr + 1)) N.zero in
  Array.blit array 0 out 0 (Array.length array);
  out.(addr) <- v;
  out

let number text =
  let text = String.trim text in
  N.of_string
    (if String.starts_with ~prefix:"+" text then
       String.sub text 1 (String.length text - 1)
     else text)

let lines text =
  let text = Utils.Lexer.without_comments text in
  List.filter (( <> ) "")
    (List.map String.trim (String.split_on_char '\n' text))

let parse_numbers text =
  Utils.Lexer.protect (fun () ->
      List.map (fun n -> value (number n)) (lines text))

let print_numbers ns = String.concat "\n" (List.map N.to_string ns)

let parse_image text =
  Utils.Lexer.protect (fun () ->
      match lines text with
      | [] -> failwith "Expected first line with 8 registers"
      | reg_line :: mem ->
          let parts =
            List.filter (( <> ) "")
              (List.map String.trim (String.split_on_char ',' reg_line))
          in
          if List.length parts <> 8 then
            failwith "Expected 8 registers on the first line (comma separated)";
          ( Array.of_list (List.map (fun s -> value (number s)) parts),
            List.map (fun s -> value (number s)) mem ))

let print_image regs mem =
  String.concat ", " (List.map N.to_string (Array.to_list regs))
  ^ if mem = [] then "" else "\n" ^ print_numbers mem

let validate_regs regs =
  if Array.length regs <> 8 then failwith "Expected 8 registers"

let decode word =
  Utils.Lexer.protect (fun () ->
      let imm, op = value (N.divmod word (N.of_int 65536)) in
      let op = value (N.to_int op) in
      let top = op lsr 8
      and mid = (op lsr 6) land 3
      and d = (op lsr 3) land 7
      and s = op land 7 in
      let cf = mid land 1 <> 0 in
      match (top, mid, d, s) with
      | 0, 0, 0, 0 -> Nop
      | 0, 1, 0, 0 -> Halt
      | 0, 0, 7, 7 -> Rst
      | 1, 0, d, s -> Ld (d, s)
      | 2, 0, d, s -> St (d, s)
      | 8, (0 | 1), d, s -> Mov (cf, d, s)
      | 8, (2 | 3), d, 0 -> Ldi (cf, d, imm)
      | 4, (0 | 1), d, s -> Add (cf, d, s)
      | 5, (0 | 1), d, s -> Sub (cf, d, s)
      | 4, (2 | 3), d, 0 -> Addi (cf, d, imm)
      | 5, (2 | 3), d, 0 -> Subi (cf, d, imm)
      | 17, 0, d, s -> Eq (d, s)
      | 18, 0, d, s -> Lt (d, s)
      | 20, 0, d, s -> Gt (d, s)
      | _ -> failwith (Printf.sprintf "Invalid instruction encoding: 0x%04x" op))

let encode inst =
  let cf flag = if flag then 1 else 0 in
  let rr top mid d s =
    N.of_int ((top lsl 8) lor (mid lsl 6) lor (d lsl 3) lor s)
  in
  let ri top flag d imm =
    N.add (rr top (2 + cf flag) d 0) (N.mul imm (N.of_int 65536))
  in
  match inst with
  | Nop -> N.zero
  | Halt -> N.of_int 64
  | Rst -> N.of_int 63
  | Ld (d, s) -> rr 1 0 d s
  | St (d, s) -> rr 2 0 d s
  | Mov (f, d, s) -> rr 8 (cf f) d s
  | Ldi (f, d, n) -> ri 8 f d n
  | Add (f, d, s) -> rr 4 (cf f) d s
  | Sub (f, d, s) -> rr 5 (cf f) d s
  | Addi (f, d, n) -> ri 4 f d n
  | Subi (f, d, n) -> ri 5 f d n
  | Eq (d, s) -> rr 17 0 d s
  | Lt (d, s) -> rr 18 0 d s
  | Gt (d, s) -> rr 20 0 d s

let execute ~resolve inst regs flag memory =
  validate_regs regs;
  let regs = Array.copy regs in
  let advance () = regs.(0) <- N.succ regs.(0) in
  let assign d n =
    regs.(d) <- n;
    if d <> 0 then advance ()
  in
  let arithmetic f d rhs op =
    if f && not flag then advance () else assign d (op regs.(d) (rhs ()))
  in
  match inst with
  | Halt -> (regs, flag, memory, true)
  | Nop ->
      advance ();
      (regs, flag, memory, false)
  | Rst ->
      advance ();
      (regs, false, memory, false)
  | Ld (d, b) ->
      let n = read memory (value (N.to_int regs.(b))) in
      assign d n;
      (regs, flag, memory, false)
  | St (s, b) ->
      let memory = write memory (value (N.to_int regs.(b))) regs.(s) in
      advance ();
      (regs, flag, memory, false)
  | Mov (f, d, s) ->
      if f && not flag then advance () else assign d regs.(s);
      (regs, flag, memory, false)
  | Ldi (f, d, n) ->
      if f && not flag then advance () else assign d (resolve n);
      (regs, flag, memory, false)
  | Add (f, d, s) ->
      arithmetic f d (fun () -> regs.(s)) N.add;
      (regs, flag, memory, false)
  | Sub (f, d, s) ->
      arithmetic f d (fun () -> regs.(s)) N.sub;
      (regs, flag, memory, false)
  | Addi (f, d, n) ->
      arithmetic f d (fun () -> resolve n) N.add;
      (regs, flag, memory, false)
  | Subi (f, d, n) ->
      arithmetic f d (fun () -> resolve n) N.sub;
      (regs, flag, memory, false)
  | Eq (d, s) ->
      let flag = N.equal regs.(d) regs.(s) in
      advance ();
      (regs, flag, memory, false)
  | Lt (d, s) ->
      let flag = N.compare regs.(d) regs.(s) < 0 in
      advance ();
      (regs, flag, memory, false)
  | Gt (d, s) ->
      let flag = N.compare regs.(d) regs.(s) > 0 in
      advance ();
      (regs, flag, memory, false)

let inst_text show = function
  | Nop -> "Nop"
  | Halt -> "Halt"
  | Rst -> "Rst"
  | Ld (d, b) -> Printf.sprintf "Ld { rd: %d, rb: %d }" d b
  | St (s, b) -> Printf.sprintf "St { rs: %d, rb: %d }" s b
  | Mov (f, d, s) ->
      Printf.sprintf "Mov { cond_flag: %b, rd: %d, rs: %d }" f d s
  | Ldi (f, d, n) ->
      Printf.sprintf "Ldi { cond_flag: %b, rd: %d, imm: %s }" f d (show n)
  | Add (f, d, s) ->
      Printf.sprintf "Add { cond_flag: %b, rd: %d, rs: %d }" f d s
  | Sub (f, d, s) ->
      Printf.sprintf "Sub { cond_flag: %b, rd: %d, rs: %d }" f d s
  | Addi (f, d, n) ->
      Printf.sprintf "Addi { cond_flag: %b, rd: %d, imm: %s }" f d (show n)
  | Subi (f, d, n) ->
      Printf.sprintf "Subi { cond_flag: %b, rd: %d, imm: %s }" f d (show n)
  | Eq (d, s) -> Printf.sprintf "Eq { rd: %d, rs: %d }" d s
  | Lt (d, s) -> Printf.sprintf "Lt { rd: %d, rs: %d }" d s
  | Gt (d, s) -> Printf.sprintf "Gt { rd: %d, rs: %d }" d s

let table title columns rows =
  Utils.Render.table ~title
    ~columns:(List.map Utils.Render.text columns)
    (List.map (List.map Utils.Render.text) rows)

let registers regs =
  table "registers" [ "reg"; "value" ]
    (Array.to_list
       (Array.mapi (fun i n -> [ "r" ^ string_of_int i; N.to_string n ]) regs))
