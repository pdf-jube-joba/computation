open Utils_runtime.Machine
open Rec_tm_shared.Syntax
open Rec_tm_shared.Environment
open Rec_tm_shared.Eval
module J = Rec_tm_ir_jump.Main
module T = Turing_machine.Main
module L = Utils_parsing.Parser
module Names = Utils_names.Identifier.Set

module State_key = struct
  type t = int * T.sign list

  let compare = Stdlib.compare
end

module States = Utils_names.Intern.Make (State_key) (T.State_id)

(* Retain only values which a later instruction can read. Function expansion
   creates fresh variables for each call site, so enumerating every valuation
   would create exponentially many unreachable states. The generated machine
   still performs precisely one tape transition for each IR instruction. *)
let live_variables body =
  let body = Array.of_list body in
  let n = Array.length body in
  let live = Array.make (n + 1) Names.empty in
  let changed = ref true in
  while !changed do
    changed := false;
    for pc = n - 1 downto 0 do
      let uses, defines, successors =
        match body.(pc) with
        | J.Assign (dst, src) -> (vars_rvalue src, vars_lvalue dst, [ pc + 1 ])
        | J.Jump (target, cond) ->
            ( vars_condition cond,
              [],
              if cond = None then [ target ] else [ target; pc + 1 ] )
        | _ -> ([], [], [ pc + 1 ])
      in
      let out =
        List.fold_left
          (fun set pc -> Names.union set live.(pc))
          Names.empty successors
      in
      let next =
        Names.union (Names.of_list uses)
          (Names.diff out (Names.of_list defines))
      in
      if not (Names.equal next live.(pc)) then (
        changed := true;
        live.(pc) <- next)
    done
  done;
  live

let compile (p : J.code) =
  let* () = J.validate p in
  L.protect (fun () ->
      let alphabet = normalized_alphabet p.alphabet in
      let vars = J.collect_vars p.body in
      let live = live_variables p.body in
      let instructions = Array.of_list p.body in
      let canonical pc values =
        List.map2
          (fun name value -> if Names.mem name live.(pc) then value else None)
          vars values
      in
      let states = ref States.empty in
      let names = ref T.Names.empty and state_names = ref [] in
      let work = Queue.create () in
      let state_for pc values =
        let values = canonical pc values in
        match States.find (pc, values) !states with
        | Some id -> id
        | None ->
            let id = States.take (pc, values) states in
            let name, next = T.Names.assign ~hint:"q" id !names in
            names := next;
            state_names := (id, name) :: !state_names;
            Queue.add (pc, values, id) work;
            id
      in
      let init_state = state_for 0 (List.map (fun _ -> None) vars) in
      let code = ref [] and accepted = ref [] in
      while not (Queue.is_empty work) do
        let pc, values, state = Queue.take work in
        if pc = Array.length instructions then accepted := state :: !accepted
        else
          let env = env_of_named (List.combine vars values) in
          List.iter
            (fun head ->
              let tape = { T.Tape.blank with head } in
              let next_pc, next_values, next_sign, dir =
                match instructions.(pc) with
                | J.Lt -> (pc + 1, values, head, T.Left)
                | J.Rt -> (pc + 1, values, head, T.Right)
                | J.Assign (dst, src) -> (
                    let value = eval env tape src in
                    match dst with
                    | Head -> (pc + 1, values, value, T.Constant)
                    | Var name ->
                        ( pc + 1,
                          List.map2
                            (fun var old -> if var = name then value else old)
                            vars values,
                          head,
                          T.Constant ))
                | J.Jump (target, c) ->
                    ( (if eval_cond env tape c then target else pc + 1),
                      values,
                      head,
                      T.Constant )
              in
              let next_state = state_for next_pc next_values in
              code := ((head, state), (next_sign, next_state, dir)) :: !code)
            alphabet
      done;
      (* A program which never reaches its end still needs a printable accept-state
     line, and this unreachable state carries no transitions. *)
      let accepted =
        if !accepted = [] then
          [
            state_for
              (Array.length instructions)
              (List.map (fun _ -> None) vars);
          ]
        else List.rev !accepted
      in
      T.unwrap
        (T.Code.of_ids ~state_names:(List.rev !state_names) init_state accepted
           (List.rev !code)))

module Source = J
module Target = T

let encode_ainput t = Ok t
let encode_rinput () = Ok ()
let decode_routput () = Ok ()
let decode_foutput t = Ok t

let api =
  Utils_runtime.Compiler_text_api.make
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
