open Instruction

let value = Utils_base.Error.get

let decode word =
  Utils_base.Error.protect (fun () ->
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
