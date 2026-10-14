open Instruction
open Memory

let value = Utils_base.Error.get

let execute ~resolve inst regs flag memory =
  Registers.validate regs;
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
