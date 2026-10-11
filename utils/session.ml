open Machine

module Make (M : S) = struct
  type t = { mutable state : M.state option }

  let create () = { state = None }

  let make session code ainput =
    let* code = M.parse_code code in
    let* ainput = M.parse_ainput ainput in
    let* state = M.make code ainput in
    session.state <- Some state;
    Ok `Null

  let with_state session f =
    match session.state with
    | None -> Error "No active machine; call make first"
    | Some state -> f state

  let step session input =
    with_state session (fun state ->
        let* input = M.parse_rinput input in
        let* result = M.step state input in
        match result with
        | Continue { next; output } ->
            session.state <- Some next;
            Ok
              (`Assoc
                 [
                   ("kind", `String "continue");
                   ("output", M.routput_json output);
                 ])
        | Halt output ->
            session.state <- None;
            Ok
              (`Assoc
                 [ ("kind", `String "halt"); ("output", M.foutput_json output) ]))

  let snapshot session = with_state session (fun state -> Ok (M.snapshot state))

  let restore session json =
    let* state = M.restore json in
    session.state <- Some state;
    Ok `Null

  let render session = with_state session (fun state -> Ok (M.render state))
end
