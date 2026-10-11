open Utils.Machine
open Ir_common
module T = Turing_machine
module L = Utils.Lexer

type lvalue = Ir_common.lvalue = Var of Utils.Identifier.t | Head
[@@deriving yojson]

type rvalue = Ir_common.rvalue =
  | Var of Utils.Identifier.t
  | Head
  | Const of T.sign
[@@deriving yojson]

type condition = Ir_common.condition = { left : rvalue; right : rvalue }
[@@deriving yojson]

type stmt =
  | Lt
  | Rt
  | Assign of lvalue * rvalue
  | Jump of int * condition option
[@@deriving yojson]

type code = { alphabet : T.sign list; body : stmt list } [@@deriving yojson]
type ainput = T.tape
type rinput = unit
type routput = unit
type foutput = T.tape

type state = { program : code; pc : int; env : environment; tape : T.tape }
[@@deriving yojson]

module Code = struct
  type t = code

  let keywords = [ "alphabet"; "jump"; "if"; "const"; "LT"; "RT" ]

  let parse =
    L.parse (fun p ->
        let alphabet = alphabet p in
        let stmt () =
          if L.eat_keyword p "LT" then Lt
          else if L.eat_keyword p "RT" then Rt
          else if L.eat_keyword p "jump" then
            let c = condition ~keywords p in
            let target = L.integer p in
            Jump (target, c)
          else
            let dst = lvalue ~keywords p in
            L.expect p ":=";
            let src = rvalue ~keywords p in
            Assign (dst, src)
        in
        let rec body () =
          if L.eof p then []
          else
            let s = stmt () in
            s :: body ()
        in
        { alphabet; body = body () })

  let stmt_print = function
    | Lt -> "LT"
    | Rt -> "RT"
    | Assign (dst, src) -> lvalue_print dst ^ " := " ^ rvalue_print src
    | Jump (target, c) ->
        "jump" ^ condition_print c ^ " " ^ string_of_int target

  let print p =
    alphabet_print p.alphabet
    ^ String.concat "" (List.map (fun s -> stmt_print s ^ "\n") p.body)
end

let collect_vars body =
  List.sort_uniq String.compare
    (List.concat_map
       (function
         | Assign (dst, src) -> vars_lvalue dst @ vars_rvalue src
         | Jump (_, c) -> vars_condition c
         | _ -> [])
       body)

let validate p =
  L.protect (fun () ->
      let allowed = normalized_alphabet p.alphabet in
      List.iteri
        (fun i -> function
          | Assign (_, src) -> validate_rvalue allowed src
          | Jump (target, c) ->
              (* The end position is the halt continuation emitted by IR lowering. *)
              if target < 0 || target > List.length p.body then
                failwith
                  (Printf.sprintf "jump target out of range at %d: %d" i target);
              validate_cond allowed c
          | _ -> ())
        p.body)

let make program tape =
  let* () = validate program in
  L.protect (fun () ->
      validate_tape (normalized_alphabet program.alphabet) tape;
      { program; pc = 0; env = env_create (collect_vars program.body); tape })

let step s () =
  L.protect (fun () ->
      if s.pc >= List.length s.program.body then Halt s.tape
      else
        let next =
          match List.nth s.program.body s.pc with
          | Lt -> { s with pc = s.pc + 1; tape = T.Tape.move_to s.tape T.Left }
          | Rt -> { s with pc = s.pc + 1; tape = T.Tape.move_to s.tape T.Right }
          | Assign (dst, src) ->
              let env, tape = assign s.env s.tape dst (eval s.env s.tape src) in
              { s with pc = s.pc + 1; env; tape }
          | Jump (target, c) ->
              if target < 0 || target > List.length s.program.body then
                failwith "jump target out of range";
              {
                s with
                pc = (if eval_cond s.env s.tape c then target else s.pc + 1);
              }
        in
        Continue { next; output = () })

let snapshot = state_to_yojson

let restore json =
  let* s = state_of_yojson json in
  let* () = validate s.program in
  L.protect (fun () ->
      if s.pc < 0 || s.pc > List.length s.program.body then
        failwith "Invalid pc";
      validate_tape (normalized_alphabet s.program.alphabet) s.tape;
      s)

let render s =
  Utils.Render.state
    [
      Utils.Render.text ~title:"pc" (string_of_int s.pc);
      Utils.Render.text ~title:"next"
        (if s.pc >= List.length s.program.body then "halt"
         else Code.stmt_print (List.nth s.program.body s.pc));
      Utils.Render.assoc_table ~title:"env" T.Sign.print s.env;
      T.Tape.render s.tape;
    ]

module Ainput = Ir_common.Tape
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput

module Machine = struct
  module Code = Code
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec state = state

  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
