open Machine

type step = Continued of string | Halted of string

module Make (M : S) = struct
  type t = { mutable state : M.state option }

  let create () = { state = None }

  let make session code ainput =
    let* code = M.Code.parse code in
    let* ainput = M.Ainput.parse ainput in
    let* state = M.make code ainput in
    session.state <- Some state;
    Ok ()

  let with_state session f =
    match session.state with
    | None -> Error "No active machine; call make first"
    | Some state -> f state

  let step session input =
    with_state session (fun state ->
        let* input = M.Rinput.parse input in
        let* result = M.step state input in
        match result with
        | Continue { next; output } ->
            session.state <- Some next;
            Ok (Continued (M.Routput.print output))
        | Halt output ->
            session.state <- None;
            Ok (Halted (M.Foutput.print output)))

  let snapshot session = with_state session (fun state -> Ok state)

  let restore session state =
    session.state <- Some state;
    Ok ()

  let render session = with_state session (fun state -> Ok (M.render state))
end
