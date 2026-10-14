include Runtime_types
module Code = Code
module Foutput = Output_text
module L = Utils_base.Error

let value = Utils_base.Error.get

let make code agents =
  Utils_base.Error.protect (fun () ->
      validate code;
      if agents <= 0 then failwith "agent count must be positive";
      {
        code;
        env = { vars = []; task_ids = [] };
        workers = Array.make agents None;
        queue = [ value (Task_id.of_int 0) ];
        tasks =
          [|
            [
              { function_name = Utils_names.Identifier.make_exn "main"; pc = 0 };
            ];
          |];
      })

let step = Scheduler.step
let snapshot = state_to_yojson

let restore json =
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

let render = View.render

module Ainput = Utils_codec.Text.Natural_int
module Rinput = Utils_codec.Text.Natural_int
module Routput = Utils_codec.Text.String
