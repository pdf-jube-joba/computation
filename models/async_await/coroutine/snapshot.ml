open Runtime_types
open Validation
module L = Utils_base.Error

let encode = state_to_yojson

let decode json =
  Result.bind (state_of_yojson json) (fun s ->
      L.protect (fun () ->
          validate s.code;
          let check id =
            if Task_id.to_int id >= Array.length s.tasks then
              failwith "Unknown task ID in snapshot"
          in
          List.iter check s.queue;
          Array.iter (Option.iter check) s.workers;
          s))
