open Runtime_types
module L = Utils_base.Error

let value = Utils_base.Error.get

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
  | Atom (Var n) -> Utils_base.Assoc.get ~default:N.zero n env.vars
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

let find_function code name =
  match List.find_opt (fun f -> f.name = name) code with
  | Some f -> f
  | None ->
      failwith ("unknown function: " ^ Utils_names.Identifier.to_string name)

let execute s ~tasks ~workers ~agent ~id ~shift stmt =
  match stmt with
  | Assign (n, a) ->
      let v = eval_aexp s.env a in
      shift 1;
      {
        s with
        env = { s.env with vars = Utils_base.Assoc.set_sorted n v s.env.vars };
      }
  | IfGoto (b, offset) ->
      shift (if eval_bexp s b then offset else 1);
      s
  | Goto offset ->
      shift offset;
      s
  | Call n ->
      ignore (find_function s.code n);
      shift 1;
      tasks.(Task_id.to_int id) <-
        { function_name = n; pc = 0 } :: tasks.(Task_id.to_int id);
      s
  | Run (n, id_var) ->
      ignore (find_function s.code n);
      let new_id = value (Task_id.of_int (Array.length tasks)) in
      shift 1;
      {
        s with
        env =
          {
            s.env with
            task_ids =
              Utils_base.Assoc.set_sorted id_var (Some new_id) s.env.task_ids;
          };
        tasks = Array.append tasks [| [ { function_name = n; pc = 0 } ] |];
        queue = s.queue @ [ new_id ];
      }
  | Yield ->
      shift 1;
      workers.(agent) <- None;
      { s with queue = s.queue @ [ id ] }

let step s agent =
  L.protect (fun () ->
      if agent < 0 || agent >= Array.length s.workers then
        failwith (Printf.sprintf "agent %d out of range" agent);
      let workers = Array.copy s.workers and tasks = Array.copy s.tasks in
      let s = { s with workers; tasks } in
      let continue s output =
        Utils_runtime.Machine.Continue { next = s; output }
      in
      match workers.(agent) with
      | None -> (
          match s.queue with
          | [] -> Utils_runtime.Machine.Halt s.env
          | id :: queue ->
              workers.(agent) <- Some id;
              continue { s with queue }
                (Printf.sprintf "agent %d: dequeue task %d" agent
                   (Task_id.to_int id)))
      | Some id -> (
          if Task_id.to_int id >= Array.length tasks then
            failwith (Printf.sprintf "unknown task %d" (Task_id.to_int id));
          match tasks.(Task_id.to_int id) with
          | [] ->
              workers.(agent) <- None;
              continue s
                (Printf.sprintf "agent %d: release task %d" agent
                   (Task_id.to_int id))
          | frame :: rest ->
              let fn = find_function s.code frame.function_name in
              if frame.pc = List.length fn.body then (
                tasks.(Task_id.to_int id) <- rest;
                continue s
                  (Printf.sprintf "agent %d: return from %s" agent
                     (Utils_names.Identifier.to_string frame.function_name)))
              else
                let stmt =
                  match List.nth_opt fn.body frame.pc with
                  | Some x -> x
                  | None ->
                      failwith
                        (Printf.sprintf "pc %d out of bounds in %s" frame.pc
                           (Utils_names.Identifier.to_string frame.function_name))
                in
                let shift offset =
                  let pc = frame.pc + offset in
                  if pc < 0 then
                    failwith
                      (Printf.sprintf "pc became negative in %s: %d + %d"
                         (Utils_names.Identifier.to_string frame.function_name)
                         frame.pc offset);
                  tasks.(Task_id.to_int id) <- { frame with pc } :: rest
                in
                let s = execute s ~tasks ~workers ~agent ~id ~shift stmt in
                continue s
                  (Printf.sprintf "agent %d: %s" agent (Code.stmt_to_text stmt))
          ))
