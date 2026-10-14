open Types

let ( let* ) = Result.bind

(* The previous snapshot format used named states in both code and runtime. *)
module Named_snapshot = struct
  type entry =
    (sign * Utils_names.Identifier.t)
    * (sign * Utils_names.Identifier.t * direction)
  [@@deriving yojson]

  type definition = {
    init_state : Utils_names.Identifier.t;
    accepted_state : Utils_names.Identifier.t list;
    code : entry list;
  }
  [@@deriving yojson]

  type t = {
    definition : definition;
    current_state : Utils_names.Identifier.t;
    tape : tape;
  }
  [@@deriving yojson]
end

let encode = state_to_yojson

let decode json =
  let* s =
    match json with
    | `Assoc fields
      when match List.assoc_opt "current_state" fields with
           | Some (`String _) -> true
           | _ -> false ->
        let* old = Named_snapshot.of_yojson json in
        let* definition =
          Code.make old.definition.init_state old.definition.accepted_state
            old.definition.code
        in
        let* current_state =
          match
            List.find_opt
              (fun (_, name) -> name = old.current_state)
              definition.state_names
          with
          | Some (id, _) -> Ok id
          | None -> Error "Unknown current state name"
        in
        Ok { definition; current_state; tape = old.tape }
    | _ -> state_of_yojson json
  in
  let* () = Code.validate s.definition in
  let* () =
    if List.mem_assoc s.current_state s.definition.state_names then Ok ()
    else Error "Unknown current state ID"
  in
  let* _ = Tape.parse (Tape.print s.tape) in
  Ok s
