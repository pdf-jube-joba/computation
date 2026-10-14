module N = Utils_base.Number

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
