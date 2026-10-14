open Runtime_types

let get_task_id env name =
  match List.assoc_opt name env.task_ids with
  | Some (Some id) -> id
  | Some None ->
      failwith
        ("task id $"
        ^ Utils_names.Identifier.to_string name
        ^ " is not initialized")
  | None ->
      failwith ("unknown task id $" ^ Utils_names.Identifier.to_string name)

let rec eval_aexp env = function
  | Atom (Var n) -> Option.value ~default:N.zero (List.assoc_opt n env.vars)
  | Atom (Imm n) -> n
  | Bin (a, op, b) -> (
      let a = eval_aexp env a in
      let b = eval_aexp env b in
      match op with Add -> N.add a b | Sub -> N.sub a b | Mul -> N.mul a b)

let rec eval_bexp s = function
  | Not b -> not (eval_bexp s b)
  | Rel (a, op, b) -> (
      let a = eval_aexp s.env a and b = eval_aexp s.env b in
      match op with
      | Eq -> N.equal a b
      | Ne -> not (N.equal a b)
      | Lt -> N.compare a b < 0)
  | And (a, b) -> eval_bexp s a && eval_bexp s b
  | Or (a, b) -> eval_bexp s a || eval_bexp s b
  | Done n ->
      let id = Task_id.to_int (get_task_id s.env n) in
      id < Array.length s.tasks && s.tasks.(id) = []
