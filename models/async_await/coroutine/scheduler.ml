open Runtime_types
module L = Utils_base.Error

let value = Utils_base.Error.get
let find_function = Task_execution.find_function

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
                let s =
                  Task_execution.execute s ~tasks ~workers ~agent ~id ~shift
                    stmt
                in
                continue s
                  (Printf.sprintf "agent %d: %s" agent (Code.stmt_to_text stmt))
          ))
