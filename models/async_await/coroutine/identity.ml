module Task_id = Utils_names.Id.Make ()

let task_text id = string_of_int (Task_id.to_int id)
