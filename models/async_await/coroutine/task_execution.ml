open Runtime_types
open Eval

let value = Utils_base.Error.get

let find_function code name =
  match List.find_opt (fun f -> f.name = name) code with
  | Some f -> f
  | None -> failwith ("unknown function: " ^ name)

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
